use clock::ChainEpoch;
use forest_address::Address;
use forest_crypto::signature::Signature;
use forest_encoding::tuple::*;
use forest_encoding::Cbor;
use forest_vm::{MethodNum, Serialized, TokenAmount, METHOD_CONSTRUCTOR};
use num_bigint::{bigint_ser, biguint_ser, BigInt};

/// Maximum number of lanes in a channel
pub const LANE_LIMIT: usize = 256;

// TODO replace placeholder when params finished
pub const SETTLE_DELAY: ChainEpoch = 1;

/// Constructor parameters for payment channel actor
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct ConstructorParams {
    pub from: Address,
    pub to: Address,
}

/// A given payment channel actor is established by `from`
/// to enable off-chain microtransactions to `to` address
/// to be reconciled and tallied on chain.
#[derive(Debug, Serialize_tuple, Deserialize_tuple)]
pub struct State {
    /// Channel owner, who has funded the actor.
    pub from: Address,
    /// Recipient of payouts from channel.
    pub to: Address,
    /// Amount successfully redeemed through the payment channel, paid out on `Collect`.
    #[serde(with = "biguint_ser")]
    pub to_send: TokenAmount,
    /// Height at which the channel can be collected.
    pub settling_at: ChainEpoch,
    /// Height before which the channel `ToSend` cannot be collected.
    pub min_settle_height: ChainEpoch,
    /// Collections of lane states for the channel, maintained in ID order.
    pub lane_states: Vec<LaneState>,
}

impl State {
    pub fn new(from: Address, to: Address) -> Self {
        Self {
            from,
            to,
            to_send: Default::default(),
            settling_at: 0,
            min_settle_height: 0,
            lane_states: Vec::new(),
        }
    }
}

/// The Lane state tracks the latest (highest) voucher nonce used to merge the lane
/// as well as the amount it has already redeemed.
#[derive(Default, Debug, Serialize_tuple, Deserialize_tuple)]
pub struct LaneState {
    /// Identifier unique to this channel
    pub id: u64,
    // TODO this could possibly be a BigUint, but won't affect serialization
    #[serde(with = "bigint_ser")]
    pub redeemed: BigInt,
    pub nonce: u64,
}

/// Specifies which `lane`s to be merged with what `nonce` on `channel_update`
#[derive(Default, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct Merge {
    pub lane: u64,
    pub nonce: u64,
}

impl Cbor for State {}
impl Cbor for LaneState {}
impl Cbor for Merge {}

/// A voucher is sent by `from` to `to` off-chain in order to enable
/// `to` to redeem payments on-chain in the future
#[derive(Default, Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct SignedVoucher {
    /// Min epoch before which the voucher cannot be redeemed
    pub time_lock_min: ChainEpoch,
    /// Max epoch beyond which the voucher cannot be redeemed
    /// set to 0 means no timeout
    pub time_lock_max: ChainEpoch,
    /// (optional) Used by `to` to validate
    // TODO revisit this type, can probably be a 32 byte array
    #[serde(with = "serde_bytes")]
    pub secret_pre_image: Vec<u8>,
    /// (optional) Specified by `from` to add a verification method to the voucher
    pub extra: Option<ModVerifyParams>,
    /// Specifies which lane the Voucher merges into (will be created if does not exist)
    pub lane: u64,
    /// Set by `from` to prevent redemption of stale vouchers on a lane
    pub nonce: u64,
    /// Amount voucher can be redeemed for
    #[serde(with = "bigint_ser")]
    pub amount: BigInt,
    /// (optional) Can extend channel min_settle_height if needed
    pub min_settle_height: ChainEpoch,

    /// (optional) Set of lanes to be merged into `lane`
    pub merges: Vec<Merge>,

    /// Sender's signature over the voucher (sign on none)
    pub signature: Option<Signature>,
}

/// Modular Verification method
#[derive(Debug, PartialEq, Serialize_tuple, Deserialize_tuple)]
pub struct ModVerifyParams {
    pub actor: Address,
    pub method: MethodNum,
    pub data: Serialized,
}

/// Payment Verification parameters
#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct PaymentVerifyParams {
    pub extra: Serialized,
    // TODO revisit these to see if they should be arrays or optional
    #[serde(with = "serde_bytes")]
    pub proof: Vec<u8>,
}

#[derive(Serialize_tuple, Deserialize_tuple)]
pub struct UpdateChannelStateParams {
    pub sv: SignedVoucher,
    #[serde(with = "serde_bytes")]
    pub secret: Vec<u8>,
    #[serde(with = "serde_bytes")]
    pub proof: Vec<u8>,
}

/// Methods payment channel
/// https://github.com/filecoin-project/specs-actors/blob/master/actors/builtin/methods.go#L49
#[repr(u64)]
pub enum MethodsPaych {
    Constructor = METHOD_CONSTRUCTOR,
    UpdateChannelState = 2,
    Settle = 3,
    Collect = 4,
}
