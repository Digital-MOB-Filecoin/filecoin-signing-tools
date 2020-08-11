use crate::error::SignerError;
use crate::signature::Signature;
use extras::{multisig, paych, ExecParams};
use forest_address::{Address, Network};
use forest_cid::{multihash::Identity, Cid, Codec};
use forest_message::{Message, SignedMessage, UnsignedMessage};
use forest_vm::Serialized;
use num_bigint_chainsafe::BigUint;
use serde::{Deserialize, Serialize, Serializer, Deserializer};
use serde::ser::SerializeSeq;
use std::convert::TryFrom;
use std::str::FromStr;
use forest_encoding::tuple::*;

pub enum SigTypes {
    SigTypeSecp256k1 = 0x01,
    SigTypeBLS = 0x02,
}

#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConstructorParamsMultisig {
    #[serde(alias = "Signers")]
    pub signers: Vec<String>,
    #[serde(alias = "NumApprovalsThreshold")]
    pub num_approvals_threshold: i64,
    #[serde(alias = "UnlockDuration")]
    pub unlock_duration: i64,
}

impl TryFrom<ConstructorParamsMultisig> for multisig::ConstructorParams {
    type Error = SignerError;

    fn try_from(
        constructor_params: ConstructorParamsMultisig,
    ) -> Result<multisig::ConstructorParams, Self::Error> {
        let signers_tmp: Result<Vec<Address>, _> = constructor_params
            .signers
            .into_iter()
            .map(|address_string| Address::from_str(&address_string))
            .collect();

        let signers = match signers_tmp {
            Ok(signers) => signers,
            Err(_) => {
                return Err(SignerError::GenericString(
                    "Failed to parse one of the signer addresses".to_string(),
                ));
            }
        };

        Ok(multisig::ConstructorParams {
            signers: signers,
            num_approvals_threshold: constructor_params.num_approvals_threshold,
            unlock_duration: constructor_params.unlock_duration,
        })
    }
}

#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExecParamsAPI {
    #[serde(alias = "CodeCid")]
    pub code_cid: String,
    #[serde(alias = "ConstructorParams")]
    pub constructor_params: String,
}

impl TryFrom<ExecParamsAPI> for ExecParams {
    type Error = SignerError;

    fn try_from(exec_constructor: ExecParamsAPI) -> Result<ExecParams, Self::Error> {
        let serialized_constructor_multisig_params =
            base64::decode(exec_constructor.constructor_params)
                .map_err(|err| SignerError::GenericString(err.to_string()))?;

        if exec_constructor.code_cid != "fil/1/multisig".to_string() {
            return Err(SignerError::GenericString(
                "Only support `fil/1/multisig` code for now.".to_string(),
            ));
        }

        Ok(ExecParams {
            code_cid: Cid::new_v1(Codec::Raw, Identity::digest(b"fil/1/multisig")),
            constructor_params: forest_vm::Serialized::new(serialized_constructor_multisig_params),
        })
    }
}

#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProposeParamsMultisig {
    #[serde(alias = "To")]
    pub to: String,
    #[serde(alias = "Value")]
    pub value: String,
    #[serde(alias = "Method")]
    pub method: u64,
    #[serde(alias = "Params")]
    pub params: String,
}

impl TryFrom<ProposeParamsMultisig> for multisig::ProposeParams {
    type Error = SignerError;

    fn try_from(propose_params: ProposeParamsMultisig) -> Result<multisig::ProposeParams, Self::Error> {
        let params = base64::decode(propose_params.params)
            .map_err(|err| SignerError::GenericString(err.to_string()))?;

        Ok(multisig::ProposeParams {
            to: Address::from_str(&propose_params.to)?,
            value: BigUint::from_str(&propose_params.value)?,
            method: propose_params.method,
            params: forest_vm::Serialized::new(params),
        })
    }
}

/// Proposal data
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PropoposalHashDataParamsMultisig {
    #[serde(alias = "Requester")]
    pub requester: String,
    #[serde(alias = "To")]
    pub to: String,
    #[serde(alias = "Value")]
    pub value: String,
    #[serde(alias = "Method")]
    pub method: u64,
    #[serde(alias = "Params")]
    pub params: String,
}

impl TryFrom<PropoposalHashDataParamsMultisig> for multisig::ProposalHashData {
    type Error = SignerError;

    fn try_from(
        proposal_params: PropoposalHashDataParamsMultisig,
    ) -> Result<multisig::ProposalHashData, Self::Error> {
        let params = base64::decode(proposal_params.params)
            .map_err(|err| SignerError::GenericString(err.to_string()))?;

        Ok(multisig::ProposalHashData {
            requester: Address::from_str(&proposal_params.requester)?,
            to: Address::from_str(&proposal_params.to)?,
            value: BigUint::from_str(&proposal_params.value)?,
            method: proposal_params.method,
            params: forest_vm::Serialized::new(params),
        })
    }
}

/// Data to approve
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TxnIDParamsMultisig {
    #[serde(alias = "TxnID")]
    pub txn_id: i64,
    #[serde(alias = "ProposalHashData")]
    pub proposal_hash_data: String,
}

impl TryFrom<TxnIDParamsMultisig> for multisig::TxnIDParams {
    type Error = SignerError;

    fn try_from(params: TxnIDParamsMultisig) -> Result<multisig::TxnIDParams, Self::Error> {
        let proposal_hash = base64::decode(params.proposal_hash_data)
            .map_err(|err| SignerError::GenericString(err.to_string()))?;

        let mut array = [0; 32];
        array.copy_from_slice(&proposal_hash);

        Ok(multisig::TxnIDParams {
            id: multisig::TxnID(params.txn_id),
            proposal_hash: array,
        })
    }
}

/// Add signer params
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AddSignerMultisigParams {
    #[serde(alias = "Signer")]
    pub signer: String,
    #[serde(alias = "Increase")]
    pub increase: bool,
}

impl TryFrom<AddSignerMultisigParams> for multisig::AddSignerParams {
    type Error = SignerError;

    fn try_from(params: AddSignerMultisigParams) -> Result<multisig::AddSignerParams, Self::Error> {
        Ok(multisig::AddSignerParams {
            signer: Address::from_str(&params.signer)?,
            increase: params.increase,
        })
    }
}

/// Remove signer params
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RemoveSignerMultisigParams {
    #[serde(alias = "Signer")]
    pub signer: String,
    #[serde(alias = "Decrease")]
    pub decrease: bool,
}

impl TryFrom<RemoveSignerMultisigParams> for multisig::RemoveSignerParams {
    type Error = SignerError;

    fn try_from(params: RemoveSignerMultisigParams) -> Result<multisig::RemoveSignerParams, Self::Error> {
        Ok(multisig::RemoveSignerParams {
            signer: Address::from_str(&params.signer)?,
            decrease: params.decrease,
        })
    }
}

/// Swap signer multisig method params
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SwapSignerMultisigParams {
    #[serde(alias = "From")]
    pub from: String,
    #[serde(alias = "To")]
    pub to: String,
}

impl TryFrom<SwapSignerMultisigParams> for multisig::SwapSignerParams {
    type Error = SignerError;

    fn try_from(params: SwapSignerMultisigParams) -> Result<multisig::SwapSignerParams, Self::Error> {
        Ok(multisig::SwapSignerParams {
            from: Address::from_str(&params.from)?,
            to: Address::from_str(&params.to)?,
        })
    }
}

/// Propose method call parameters
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChangeNumApprovalsThresholdMultisigParams {
    #[serde(alias = "NewTreshold")]
    pub new_threshold: i64,
}

impl TryFrom<ChangeNumApprovalsThresholdMultisigParams> for multisig::ChangeNumApprovalsThresholdParams {
    type Error = SignerError;

    fn try_from(
        params: ChangeNumApprovalsThresholdMultisigParams,
    ) -> Result<multisig::ChangeNumApprovalsThresholdParams, Self::Error> {
        Ok(multisig::ChangeNumApprovalsThresholdParams {
            new_threshold: params.new_threshold,
        })
    }
}

/// Payment channel create params
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaymentChannelCreateParams {
    #[serde(alias = "From")]
    pub from: String,
    #[serde(alias = "To")]
    pub to: String,
}

impl TryFrom<PaymentChannelCreateParams> for paych::ConstructorParams {
    type Error = SignerError;

    fn try_from(params: PaymentChannelCreateParams) -> Result<paych::ConstructorParams, Self::Error> {
        Ok(paych::ConstructorParams {
            from: Address::from_str(&params.from)?,
            to: Address::from_str(&params.to)?,
        })
    }
}

/// Payment channel update state params
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PaymentChannelUpdateStateParams {
    pub sv: String,
    pub secret: Vec<u8>,
    pub proof: Vec<u8>,
}

impl TryFrom<PaymentChannelUpdateStateParams> for paych::UpdateChannelStateParams {
    type Error = SignerError;

    fn try_from(params: PaymentChannelUpdateStateParams) -> Result<paych::UpdateChannelStateParams, Self::Error> {
        let cbor_sv = base64::decode(params.sv)?;
        let sv : paych::SignedVoucher = forest_encoding::from_slice(cbor_sv.as_ref())?;
        Ok(paych::UpdateChannelStateParams {
            sv: sv,
            secret: vec![],
            proof: vec![],
        })
    }
}

/// *crypto.Signature Go type:  specs-actors/actors/crytpo:Signature
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct SpecsActorsCryptoSignature {
    pub typ: u8,
    #[serde(with = "serde_bytes")]
    pub data: Vec<u8>,
}

impl From<&SpecsActorsCryptoSignature> for SpecsActorsCryptoSignature {
    fn from(sig: &SpecsActorsCryptoSignature) -> Self {
        let d = sig.data.iter().map(|el|  el.clone() ).collect();
        SpecsActorsCryptoSignature {
            typ: sig.typ,
            data: d,
        }
    }
}

impl Serialize for SpecsActorsCryptoSignature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> 
        where S: Serializer
    {
        let mut v = Vec::<u8>::new();
        v.push(self.typ);
        v.extend(self.data.iter().map(|x| x.clone() ));
        serde_bytes::Serialize::serialize(&v, serializer)
    }
}

/// Signed voucher
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
//#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[derive(Debug, Clone, PartialEq, Deserialize_tuple, Serialize_tuple)]
pub struct SignedVoucherAPI {
    pub timeLockMin: i64,                       // not supported - must be 0
    pub timeLockMax: i64,                       // not supported - must be 0
    // Serializing to 0x40 (empty byte string), consistent with Lotus-generated voucher
    #[serde(with = "serde_bytes")]
    pub secretPreimage: Vec<u8>,                // not supported - must be blank
    // This should serialize to 0xf6 (cbor null) per https://github.com/whyrusleeping/cbor-gen/blob/227fab5a237792e633ddc99254a5a8a03642cb7a/utils.go#L523
    pub extra: Option::<i32>,                   // not supported - must be None
    pub lane: u64,
    pub nonce: u64,
    // Needs to serialize as either:
    //   empty byte array                                      if Amount==0
    //   concat(0x00,[Amount encoded as a big endian slice])   if Amount>0
    // ref:  https://github.com/filecoin-project/specs-actors/blob/master/actors/abi/big/int.go#L225
    #[serde(serialize_with = "SignedVoucherAPI::serde_ser_to_cbor_be_int_bytes")]
    pub amount: u64,
    pub minSettleHeight: i64,                   // not supported - must be zero
    // Must serialize to 0x80 (cbor empty array)
    pub merges: [u8;0],
    // If this is absent, it must serialize as 0xf6 (cbor null).
    pub signature: Option<SpecsActorsCryptoSignature>,
}


impl From<&SignedVoucherAPI> for SignedVoucherAPI {
    fn from(sv: &SignedVoucherAPI) -> Self {
        let sig : SpecsActorsCryptoSignature = (&sv.signature).as_ref().unwrap().into();
        Self::new(sv.lane, sv.nonce, sv.amount, &sig)
    }
}

impl SignedVoucherAPI {
    pub fn new(lane: u64, nonce: u64, amount: u64, signature: &SpecsActorsCryptoSignature) -> SignedVoucherAPI {
        //let signature : SpecsActorsCryptoSignature = signature.into();
        SignedVoucherAPI{
            timeLockMin: 0,
            timeLockMax: 0,
            secretPreimage: vec![],
            extra: Option::<i32>::None,
            lane,
            nonce,
            amount,
            minSettleHeight: 0,
            merges: [],
            signature: Some( signature.into() )
        }
    }

    /// Serialises a u64 into its 0x00-prefixed big endian representation
    pub fn serde_ser_to_cbor_be_int_bytes<S>(n: &u64, serializer: S) -> Result<S::Ok, S::Error> 
        where S: Serializer
    {
        let be_bytes = n.to_be_bytes();
        let mut v = Vec::<u8>::new();
        v.push(0 as u8);
        let mut leading_zero = true;
        
        // v.extend(be_bytes.iter().skip_white(|&x| *x==0 ));
        // ^^^ would be cleaner but skip_while not implemented for vec<u8> so using this ugliness instead:
        v.extend(be_bytes.iter().filter_map(|x| if *x==0 {
            if !leading_zero {
                Some(*x)
            } else {
                None
            }
        } else {
            leading_zero = false;
            Some(*x)
        }));

        serde_bytes::Serialize::serialize(&v, serializer)
    }
}

#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MessageParams {
    MessageParamsSerialized(String),
    PropoposalHashDataParamsMultisig(PropoposalHashDataParamsMultisig),
    ConstructorParamsMultisig(ConstructorParamsMultisig),
    MessageParamsMultisig(ExecParamsAPI),
    ProposeParamsMultisig(ProposeParamsMultisig),
    TxnIDParamsMultisig(TxnIDParamsMultisig),
    AddSignerMultisigParams(AddSignerMultisigParams),
    RemoveSignerMultisigParams(RemoveSignerMultisigParams),
    SwapSignerMultisigParams(SwapSignerMultisigParams),
    ChangeNumApprovalsThresholdMultisigParams(ChangeNumApprovalsThresholdMultisigParams),
    PaymentChannelCreateParams(PaymentChannelCreateParams),
    PaymentChannelUpdateStateParams(PaymentChannelUpdateStateParams),
}

impl MessageParams {
    pub fn serialize(self) -> Result<Serialized, SignerError> {
        let params_serialized = match self {
            MessageParams::MessageParamsSerialized(params_string) => {
                let params_bytes = base64::decode(&params_string)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?;
                forest_vm::Serialized::new(params_bytes)
            }
            MessageParams::PropoposalHashDataParamsMultisig(params) => {
                let params = multisig::ProposalHashData::try_from(params)?;

                forest_vm::Serialized::serialize::<multisig::ProposalHashData>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::ConstructorParamsMultisig(constructor_params) => {
                let params = multisig::ConstructorParams::try_from(constructor_params)?;

                forest_vm::Serialized::serialize::<multisig::ConstructorParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::MessageParamsMultisig(multisig_params) => {
                let params = ExecParams::try_from(multisig_params)?;

                forest_vm::Serialized::serialize::<ExecParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::ProposeParamsMultisig(multisig_proposal_params) => {
                let params = multisig::ProposeParams::try_from(multisig_proposal_params)?;

                forest_vm::Serialized::serialize::<multisig::ProposeParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::TxnIDParamsMultisig(multisig_txn_id_params) => {
                let params = multisig::TxnIDParams::try_from(multisig_txn_id_params)?;

                forest_vm::Serialized::serialize::<multisig::TxnIDParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::AddSignerMultisigParams(add_signer_params) => {
                let params = multisig::AddSignerParams::try_from(add_signer_params)?;

                forest_vm::Serialized::serialize::<multisig::AddSignerParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::RemoveSignerMultisigParams(remove_signer_params) => {
                let params = multisig::RemoveSignerParams::try_from(remove_signer_params)?;

                forest_vm::Serialized::serialize::<multisig::RemoveSignerParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::SwapSignerMultisigParams(swap_signer_params) => {
                let params = multisig::SwapSignerParams::try_from(swap_signer_params)?;

                forest_vm::Serialized::serialize::<multisig::SwapSignerParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::ChangeNumApprovalsThresholdMultisigParams(
                change_num_approvals_treshold_params,
            ) => {
                let params = multisig::ChangeNumApprovalsThresholdParams::try_from(
                    change_num_approvals_treshold_params,
                )?;

                forest_vm::Serialized::serialize::<multisig::ChangeNumApprovalsThresholdParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::PaymentChannelCreateParams(pymtchan_create_params) => {
                let params = paych::ConstructorParams::try_from(pymtchan_create_params)?;

                forest_vm::Serialized::serialize::<paych::ConstructorParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }
            MessageParams::PaymentChannelUpdateStateParams(pch_update_params) => {
                let params = paych::UpdateChannelStateParams::try_from(pch_update_params)?;

                forest_vm::Serialized::serialize::<paych::UpdateChannelStateParams>(params)
                    .map_err(|err| SignerError::GenericString(err.to_string()))?
            }

        };

        Ok(params_serialized)
    }
}

/// Unsigned message api structure
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnsignedMessageAPI {
    pub to: String,
    pub from: String,
    pub nonce: u64,
    pub value: String,
    #[serde(rename = "gasprice")]
    #[serde(alias = "gasPrice")]
    #[serde(alias = "gas_price")]
    pub gas_price: String,
    #[serde(rename = "gaslimit")]
    #[serde(alias = "gasLimit")]
    #[serde(alias = "gas_limit")]
    pub gas_limit: u64,
    pub method: u64,
    pub params: String,
}

/// Signature api structure
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct SignatureAPI {
    #[serde(rename = "type")]
    pub sig_type: u8,
    #[serde(with = "serde_base64_vector")]
    pub data: Vec<u8>,
}

/// Signed message api structure
#[cfg_attr(feature = "with-arbitrary", derive(arbitrary::Arbitrary))]
#[derive(Debug, Clone, Deserialize, PartialEq, Serialize)]
pub struct SignedMessageAPI {
    pub message: UnsignedMessageAPI,
    pub signature: SignatureAPI,
}

/// Structure containing an `UnsignedMessageAPI` or a `SignedMessageAPI`
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageTxAPI {
    UnsignedMessageAPI(UnsignedMessageAPI),
    SignedMessageAPI(SignedMessageAPI),
}

impl MessageTxAPI {
    pub fn get_message(&self) -> UnsignedMessageAPI {
        match self {
            MessageTxAPI::UnsignedMessageAPI(unsigned_message_api) => {
                unsigned_message_api.to_owned()
            }
            MessageTxAPI::SignedMessageAPI(signed_message_api) => {
                signed_message_api.message.to_owned()
            }
        }
    }
}

/// Structure containing an `UnsignedMessage` or a `SignedMessage` from forest_address
#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum MessageTx {
    UnsignedMessage(UnsignedMessage),
    SignedMessage(SignedMessage),
}

/// Message structure with network parameter
pub struct MessageTxNetwork {
    pub message_tx: MessageTx,
    pub testnet: bool,
}

impl From<&Signature> for SignatureAPI {
    fn from(sig: &Signature) -> SignatureAPI {
        match sig {
            Signature::SignatureSECP256K1(sig_secp256k1) => SignatureAPI {
                sig_type: SigTypes::SigTypeSecp256k1 as u8,
                data: sig_secp256k1.0.to_vec(),
            },
            Signature::SignatureBLS(sig_bls) => SignatureAPI {
                sig_type: SigTypes::SigTypeBLS as u8,
                data: sig_bls.0.to_vec(),
            },
        }
    }
}

mod serde_base64_vector {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(v: &[u8], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&base64::encode(v))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        base64::decode(s).map_err(serde::de::Error::custom)
    }
}

impl TryFrom<MessageTxNetwork> for MessageTxAPI {
    type Error = SignerError;

    fn try_from(message_tx_network: MessageTxNetwork) -> Result<MessageTxAPI, Self::Error> {
        let network = if message_tx_network.testnet {
            Network::Testnet
        } else {
            Network::Mainnet
        };

        match message_tx_network.message_tx {
            MessageTx::UnsignedMessage(message_tx) => {
                let mut to_address: forest_address::Address = message_tx.to().to_owned();
                to_address.set_network(network);

                let mut from_address: forest_address::Address = message_tx.from().to_owned();
                from_address.set_network(network);

                let tmp = UnsignedMessageAPI::from(message_tx);

                let unsigned_message_user_api = UnsignedMessageAPI {
                    to: to_address.to_string(),
                    from: from_address.to_string(),
                    ..tmp
                };

                Ok(MessageTxAPI::UnsignedMessageAPI(unsigned_message_user_api))
            }
            MessageTx::SignedMessage(message_tx) => {
                let mut to_address: forest_address::Address = message_tx.to().to_owned();
                to_address.set_network(network);

                let mut from_address: forest_address::Address = message_tx.from().to_owned();
                from_address.set_network(network);

                let tmp = UnsignedMessageAPI::from(message_tx.message().clone());

                let unsigned_message_user_api = UnsignedMessageAPI {
                    to: to_address.to_string(),
                    from: from_address.to_string(),
                    ..tmp
                };

                let y = Signature::try_from(message_tx.signature().bytes().to_vec())?;

                let signed_message_api = SignedMessageAPI {
                    message: unsigned_message_user_api,
                    signature: SignatureAPI::from(&y),
                };

                Ok(MessageTxAPI::SignedMessageAPI(signed_message_api))
            }
        }
    }
}

impl From<MessageTx> for MessageTxAPI {
    fn from(message_tx: MessageTx) -> MessageTxAPI {
        match message_tx {
            MessageTx::UnsignedMessage(message_tx) => {
                MessageTxAPI::UnsignedMessageAPI(UnsignedMessageAPI::from(message_tx))
            }
            MessageTx::SignedMessage(message_tx) => {
                MessageTxAPI::SignedMessageAPI(SignedMessageAPI::from(message_tx))
            }
        }
    }
}

impl TryFrom<&UnsignedMessageAPI> for UnsignedMessage {
    type Error = SignerError;

    fn try_from(message_api: &UnsignedMessageAPI) -> Result<UnsignedMessage, Self::Error> {
        let to = Address::from_str(&message_api.to)
            .map_err(|err| SignerError::GenericString(err.to_string()))?;
        let from = Address::from_str(&message_api.from)
            .map_err(|err| SignerError::GenericString(err.to_string()))?;
        let value = BigUint::from_str(&message_api.value)?;
        let gas_limit = message_api.gas_limit;
        let gas_price = BigUint::from_str(&message_api.gas_price)?;

        let message_params_bytes = base64::decode(&message_api.params)
            .map_err(|err| SignerError::GenericString(err.to_string()))?;
        let params = Serialized::new(message_params_bytes);

        let tmp = UnsignedMessage::builder()
            .to(to)
            .from(from)
            .sequence(message_api.nonce)
            .value(value)
            .method_num(message_api.method)
            .params(params)
            .gas_limit(gas_limit)
            .gas_price(gas_price)
            .build()
            .map_err(SignerError::GenericString)?;

        Ok(tmp)
    }
}

impl From<UnsignedMessage> for UnsignedMessageAPI {
    fn from(unsigned_message: UnsignedMessage) -> UnsignedMessageAPI {
        let params_hex_string = hex::encode(unsigned_message.params().bytes());

        UnsignedMessageAPI {
            to: unsigned_message.to().to_string(),
            from: unsigned_message.from().to_string(),
            nonce: unsigned_message.sequence(),
            value: unsigned_message.value().to_string(),
            gas_price: unsigned_message.gas_price().to_string(),
            gas_limit: unsigned_message.gas_limit(),
            method: unsigned_message.method_num(),
            params: params_hex_string,
        }
    }
}

impl From<SignedMessage> for SignedMessageAPI {
    fn from(signed_message: SignedMessage) -> SignedMessageAPI {
        SignedMessageAPI {
            message: UnsignedMessageAPI::from(signed_message.message().clone()),
            signature: SignatureAPI {
                sig_type: SigTypes::SigTypeSecp256k1 as u8,
                data: signed_message.signature().bytes().to_vec(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::api::UnsignedMessageAPI;
    use forest_encoding::{from_slice, to_vec};
    use forest_message::UnsignedMessage;
    use hex::{decode, encode};
    use std::convert::TryFrom;

    const EXAMPLE_UNSIGNED_MESSAGE: &str = r#"
        {
            "to": "t17uoq6tp427uzv7fztkbsnn64iwotfrristwpryy",
            "from": "t1xcbgdhkgkwht3hrrnui3jdopeejsoas2rujnkdi",
            "nonce": 1,
            "value": "100000",
            "gasprice": "2500",
            "gaslimit": 25000,
            "method": 0,
            "params": ""
        }"#;

    const EXAMPLE_CBOR_DATA: &str =
        "89005501fd1d0f4dfcd7e99afcb99a8326b7dc459d32c6285501b882619d46558f3d9e316d11b48dcf211327025a0144000186a0430009c41961a80040";

    #[test]
    fn json_to_cbor() {
        let message_api: UnsignedMessageAPI =
            serde_json::from_str(EXAMPLE_UNSIGNED_MESSAGE).expect("FIXME");
        println!("{:?}", message_api);

        let message = UnsignedMessage::try_from(&message_api).expect("FIXME");

        let message_cbor: Vec<u8> = to_vec(&message).expect("Cbor serialization failed");
        let message_cbor_hex = encode(message_cbor);

        println!("{:?}", message_cbor_hex);
        assert_eq!(EXAMPLE_CBOR_DATA, message_cbor_hex)
    }

    #[test]
    fn cbor_to_json() {
        let cbor_buffer = decode(EXAMPLE_CBOR_DATA).expect("FIXME");

        let message: UnsignedMessage = from_slice(&cbor_buffer).expect("could not decode cbor");
        println!("{:?}", message);

        let message_user_api =
            UnsignedMessageAPI::try_from(message).expect("could not convert message");

        let message_user_api_json =
            serde_json::to_string_pretty(&message_user_api).expect("could not serialize as JSON");

        println!("{}", message_user_api_json);

        let message_api: UnsignedMessageAPI =
            serde_json::from_str(EXAMPLE_UNSIGNED_MESSAGE).expect("FIXME");

        assert_eq!(message_api, message_user_api)
    }
}
