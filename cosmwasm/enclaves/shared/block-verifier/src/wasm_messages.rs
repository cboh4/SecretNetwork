use alloc::collections::VecDeque;
use cosmos_proto::tx::tx::Tx;
use lazy_static::lazy_static;

use std::sync::SgxMutex;

lazy_static! {
    pub static ref VERIFIED_MESSAGES: SgxMutex<VerifiedWasmMessages> =
        SgxMutex::new(VerifiedWasmMessages::default());
}
// use cosmrs::{tx as cosmtx, Tx};
// use enclave_utils::tx_bytes::TxBytesForHeight;
// use log::error;
// use sgx_types::{sgx_status_t, SgxResult};
//
// pub fn parse_tx(raw_txs: &TxBytesForHeight) -> SgxResult<Vec<cosmrs::Tx>> {
//     let result: Result<Vec<cosmrs::Tx>, _> = raw_txs
//         .txs
//         .iter()
//         .map_ok(|tx| {
//             Tx::from_bytes(tx.tx.as_slice()).map_err(|e| {
//                 error!("Failed to parse tx");
//                 sgx_status_t::SGX_ERROR_INVALID_SIGNATURE
//             })
//         })
//         .collect();
//
//     result
// }
//
// pub struct TxForValidation {
//     current: cosmrs::Tx,
//     remaining: Vec<Tx>,
// }
//
// impl TxForValidation {
//     fn get_next_sign_bytes(&mut self) {
//         self.current.body.into_bytes()
//     }
// }
//
// lazy_static! {
//   static ref CURRENT_TX: SgxMutex = SgxMutex::new(MsgCounter::default());
// }

pub fn message_is_wasm(msg: &protobuf::well_known_types::Any) -> bool {
    matches!(
        msg.type_url.as_str(),
        "/secret.compute.v1beta1.MsgExecuteContract"
            | "/secret.compute.v1beta1.MsgInstantiateContract"
    )
}

pub fn message_is_reg(msg: &protobuf::well_known_types::Any) -> bool {
    matches!(
        msg.type_url.as_str(),
        "/secret.registration.v1beta1.RaAuthenticate"
    )
}

#[derive(Debug, Clone, Default)]
pub struct VerifiedWasmMessages {
    messages: VecDeque<Vec<u8>>,
    height: u64,
    time: i128,
}

impl VerifiedWasmMessages {
    pub fn get_next(&mut self) -> Option<Vec<u8>> {
        self.messages.pop_front()
    }

    pub fn remaining(&self) -> usize {
        self.messages.len()
    }

    pub fn append_msg_from_tx(&mut self, mut tx: Tx) {
        for msg in tx.take_body().messages {
            if message_is_wasm(&msg) | message_is_reg(&msg) {
                self.messages.push_back(msg.value);
            }
        }
    }

    pub fn set_block_info(&mut self, height: u64, time: i128) {
        self.height = height;
        self.time = time;
    }

    pub fn height(&self) -> u64 {
        self.height
    }
    pub fn time(&self) -> i128 {
        self.time
    }

    pub fn clear(&mut self) {
        self.messages.clear()
    }
}

#[cfg(feature = "test")]
pub mod tests {

    use base64;
    use cosmos_proto::tx as protoTx;
    use hex;
    use protobuf::Message;

    const TX_RAW_SINGLE_WASM_MSG: &str = "0abe020abb020a2a2f7365637265742e636f6d707574652e763162657461312e4d736745786563757465436f6e7472616374128c020a14def8f4c5de676431f1bac48c892b5e4593f3b4f312143b1a7485c6162c5883ee45fb2d7477a87d8a4ce51add01a715462d5ca8feb6dceb58e21d8794e8f0257361871006e5e51a7d5e9e136e1d908024e1ea59a1008ef28c998b60a47b1836dbf70e3e8454078db37bbd0c7e3d1d5b939622fc5989ffc119684acae9a9750d910105d5aaaa6e4e008b44765802351814f3b25626dd4a545c494d174f312453ccbc88ead428d97426cedcd080b4a88d95f929bcf66693cfc497918ba94861f877df5a280a8424bfda118afd01a581dcff7bb1e983370a047275ca03a15fd2c4582e10f141a2228137c75206a975f1934050a96abe3a97d2a21aafe7845a15003d8a8cd01c0e5560d0ac2c12520a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103fdeea779e2da196817e46ed6566eed937a00b31b3b26351fc86d7519a6ffac7f12040a02080112001a409b0cf1103b1b578fd1d4c0ea37a7ea258c39ae32918df1334c68ad18674cc1450a6433f0f693f2aee8b57e55eb184f8149e738d68cf4d3a954c43b9fab4b1f5b";
    const TX_RAW_2_WASM_MSG: &str = "0ab4040a97020a2a2f7365637265742e636f6d707574652e763162657461312e4d736745786563757465436f6e747261637412e8010a14a082110ac6b058019d436d718a4e79f70d27357212143b1a7485c6162c5883ee45fb2d7477a87d8a4ce51a9e01c7f0e5d6f46bc1fa66b4e0e439e8b7c5d89cb20f0261b3dc21e4ed31e7752ca05f652f4faed2125bccac7851d95f906c6cb36c4132b65d6c86adf76466e3543aa5e1f66fc0d9ff4780dcadebff66c163af0b93747f3d4239a9881a2295cd425b689b023a4ecbe411cd41d28826ec4c396d8faadcf6f1fdd9077b4ea24b3f4fb6f931a046d23207bafa940de07d54c009cced68545f05e1dad766d706255e2a0c0a05617373616612033133302a0b0a0564656e6f6d120231350a97020a2a2f7365637265742e636f6d707574652e763162657461312e4d736745786563757465436f6e747261637412e8010a14a082110ac6b058019d436d718a4e79f70d27357212143b1a7485c6162c5883ee45fb2d7477a87d8a4ce51a9e01c7f0e5d6f46bc1fa66b4e0e439e8b7c5d89cb20f0261b3dc21e4ed31e7752ca05f652f4faed2125bccac7851d95f906c6cb36c4132b65d6c86adf76466e3543aa5e1f66fc0d9ff4780dcadebff66c163af0b93747f3d4239a9881a2295cd425b689b023a4ecbe411cd41d28826ec4c396d8faadcf6f1fdd9077b4ea24b3f4fb6f931a046d23207bafa940de07d54c009cced68545f05e1dad766d706255e2a0c0a05617373616612033133302a0b0a0564656e6f6d1202313512a2010a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103b1aaf60dba87c43e1dc3e1b1b4f9c39f41fd9f97f9073106329d676517a482eb12040a0208010a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103b1aaf60dba87c43e1dc3e1b1b4f9c39f41fd9f97f9073106329d676517a482eb12040a02080112001a40d980b90c0b67c34568872db73ef335b8445099b433c7211f0adde85179fa74da5d655ff4f4c478b38ee2fe9da49c5f321a92268b6ebd24e81bd19deb64befa461a40d980b90c0b67c34568872db73ef335b8445099b433c7211f0adde85179fa74da5d655ff4f4c478b38ee2fe9da49c5f321a92268b6ebd24e81bd19deb64befa46";
    const TX_RAW_2_WASM_1_BANK_MSG: &str = "0ad0050a97020a2a2f7365637265742e636f6d707574652e763162657461312e4d736745786563757465436f6e747261637412e8010a1406a918f7c66a8f4182f4a6304f8600c98261484712143b1a7485c6162c5883ee45fb2d7477a87d8a4ce51a9e01f1f5895860fbfc3f849e6349b801d19c6d430c64edd37c660b18f1a82f08d3ee5f652f4faed2125bccac7851d95f906c6cb36c4132b65d6c86adf76466e3543a9ac4ec5f75c7af3318a3fd66fa1a2e8747344bf02dc0e128b05ccdbac74a8b19e22957ecf787a40928091bd39e5cd2267ec477d0e5280ae6351497601b97ec79dacf22250cd79d991d9026c17258f517cc1b864d15dc510a1bf70c8022e82a0c0a05617373616612033133302a0b0a0564656e6f6d120231350a97020a2a2f7365637265742e636f6d707574652e763162657461312e4d736745786563757465436f6e747261637412e8010a1406a918f7c66a8f4182f4a6304f8600c98261484712143b1a7485c6162c5883ee45fb2d7477a87d8a4ce51a9e01f1f5895860fbfc3f849e6349b801d19c6d430c64edd37c660b18f1a82f08d3ee5f652f4faed2125bccac7851d95f906c6cb36c4132b65d6c86adf76466e3543a9ac4ec5f75c7af3318a3fd66fa1a2e8747344bf02dc0e128b05ccdbac74a8b19e22957ecf787a40928091bd39e5cd2267ec477d0e5280ae6351497601b97ec79dacf22250cd79d991d9026c17258f517cc1b864d15dc510a1bf70c8022e82a0c0a05617373616612033133302a0b0a0564656e6f6d120231350a99010a1c2f636f736d6f732e62616e6b2e763162657461312e4d736753656e6412790a2d7365637265743171363533336137786432383572716835356363796c707371657870787a6a7a38717a6e386e38122d7365637265743171363533336137786432383572716835356363796c707371657870787a6a7a38717a6e386e381a0c0a05617373616612033133301a0b0a0564656e6f6d1202313512f2010a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103d3c85ce8007e9a745e5dc986aa721289da524c08adee427cbc58d0a0b015eaf012040a0208010a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103d3c85ce8007e9a745e5dc986aa721289da524c08adee427cbc58d0a0b015eaf012040a0208010a4e0a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2103d3c85ce8007e9a745e5dc986aa721289da524c08adee427cbc58d0a0b015eaf012040a02080112001a40d95940e2c70ed12f223dce90e1f89f357cabdc16e234467cc2534e4554c804004b8457db52af71228726b170fa7c897352eec906e9a948ef327182725f01192a1a40d95940e2c70ed12f223dce90e1f89f357cabdc16e234467cc2534e4554c804004b8457db52af71228726b170fa7c897352eec906e9a948ef327182725f01192a1a40d95940e2c70ed12f223dce90e1f89f357cabdc16e234467cc2534e4554c804004b8457db52af71228726b170fa7c897352eec906e9a948ef327182725f01192a";
    const TX_RAW_MULTISIG_1_WASM_MSG: &str = "0afe010afb010a2e2f7365637265742e636f6d707574652e763162657461312e4d7367496e7374616e7469617465436f6e747261637412c8010a146ec8bb5da9e88846080909c92e17a4ffb5a26d3d1801221164656d6f20636f6e7472616374203020302a9a01e59282e2ba371e6a2e5010143b457a0af2abfa1e841b07f564c46dd0a059821ea84badd295af30098a70c150f5a8fcdd2c064f9c4496819a901128d7fa39851c19973806608a55cb38975ef40e9759f99a4f19936b0ed03217fea6a5d04fa3c6ce3e5723e26419f8abdabcb5de5bcbec1cb07506a3251cc33116f76f1feda7aee211388b01d755749d066e535daa4c56744809a945b766feaa90128f010a8a010a770a292f636f736d6f732e63727970746f2e6d756c74697369672e4c6567616379416d696e6f5075624b6579124a080112460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a21030c8fd4f611563ded659dba55c334088ec75bbedf55b59becc93b6fb00650fe97120f120d0a05080112018012040a02087f12001a420a40c151aa0983ceaf6eea9fb634d0d05a79b23ff1fae11186de6782a768d466421534fbf572d086065819b77d59f81370dd343bac676aec3920244c7d5292be452d";

    const TX_RAW_REGISTRATION_MSG: &str = "0abb1d0ab81d0a2b2f7365637265742e726567697374726174696f6e2e763162657461312e526141757468656e74696361746512881d0a14ad4510fb2c8a82b7aba9d349b5bbaaaeda38879e12ef1c30820e6b30820e12a003020102020101300a06082a8648ce3d04030230143112301006035504030c09536563726574544545301e170d3233303532313133333332305a170d3333303532303133333332305a302a3128302606035504030c1f536563726574204e6574776f726b204e6f64652043657274696669636174653059301306072a8648ce3d020106082a8648ce3d03010703420004ced2fb7eeacd87cb5afec5013a8a81ae317d99c597472193b6a6b39b49bb6bbb7f43c4f3676eff59c1b7c8cc48d394eac9724057b3e565ec1dc0072bbe7a02f8a3820d3d30820d3930820d3506096086480186f842010d04820d267b227265706f7274223a2265794a705a434936496a45324d6a517a4d6a67774f5445334e7a55304d7a45314e6a67344d4445344e5445324e4459774d4467794e44597a4f4467794d794973496e52706257567a6447467463434936496a49774d6a4d744d4455744d6a46554d544d364d7a4d364d6a41754d6a677a4d444d3349697769646d567963326c76626949364e4377695a5842705a46427a5a58566b6232353562534936496d74426169744e63316836626d4d3561334e765357787a5956685a4b325a6c62315a4e556d746c5a555a755a4339684e4452434d5374744d3268715443737a5a53743562466c6b56464d7a51793944637a4e4357475268556e4a5a5744647a6333497855474d72516d4d3056564a714f4774736369746e654764574e336c534e46687a6369395951575a724d465a544e433831626a466f56455a334d6b746b615756435333687856335a5857564e6d6233565857584676616a67786431567655315234616d64715445526963477871634570324c304a726230746e576d74364d7a67324d4430694c434a685a485a70633239796556565354434936496d68306448427a4f6938766332566a64584a7064486b7459325675644756794c6d6c75644756734c6d4e7662534973496d466b646d6c7a62334a355355527a496a7062496b6c4f5645564d4c564e424c5441774d7a4d30496977695355355552557774553045744d4441324d5455695853776961584e325257356a624746325a5646316233526c553352686448567a496a6f6955316466534546535245564f53553548583035465255524652434973496d6c7a646b567559327868646d5652645739305a554a765a486b694f694a425a3046435155564254554642515539425154524251554642515546495a7a686b5a6a424653476c7064573930646b3554523059785a433830515546425155464251554642515546425155464251554642515546425258684e513049764b30464364304642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554a52515546425155464251554642534546425155464251554642515570505257314565454e6a5747564f4e305272616c463363304935516d593556554d305a57744c4c33466d62305978575864594d54564f576b5a42515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464452566851546b6c4755464a5762474e6a5a7974434f54426c5654453056315247535651775555597864565645596e5a70534555764f4339775a304642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425a304642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155464252474a3653444d335654524c656c5269626e52485a584578597a5a35556d35784e32355057564651623163315158646c4f455a474e446c5657476442515546425155464251554642515546425155464251554642515546425155464251554642515546425155464251554642515546425155456966513d3d222c227369676e6174757265223a22446b62314d73676f384259536b485051466e534d594a79394b554e2f44796c4e506448445734343736425868346d4c374c682b647638434664726f534b77383736455461784268552f4f702b7853552b6e78356d77434b524a463054497354434575716f734d4e556d4e50755545315241485578484c4f424239435567776447716a55727171384d49652b43642b6d727671594a6c6d397a475159706c357358795a6f6b4a6e4e35615246377a6d576a7842376e51545447313478424a332b2f4d313366596277314e4d6d686d375a424471674371774331743676313850705555582b67354d436434786a66714f2b30644c7970744e4649497349584544653358596866494c4b6b326c5230776f6b344933554861796b63434c5166704568306f316d324c6456484730484457656f79637a305959384b6c4775716a557553475378527a736d554f5843682b6c3151753266736e48513d3d222c227369676e696e675f63657274223a224d4949456f54434341776d67417749424167494a414e4548646c30796f3743574d413047435371475349623344514542437755414d483478437a414a42674e5642415954416c56544d517377435159445651514944414a44515445554d424947413155454277774c553246756447456751327868636d4578476a415942674e5642416f4d45556c756447567349454e76636e4276636d4630615739754d5441774c675944565151444443644a626e526c6243425452316767515852305a584e305958527062323467556d567762334a3049464e705a323570626d6367513045774868634e4d5459784d5449794d446b7a4e6a55345768634e4d6a59784d5449774d446b7a4e6a5534576a42374d517377435159445651514745774a56557a454c4d416b474131554543417743513045784644415342674e564241634d43314e68626e526849454e7359584a684d526f77474159445651514b4442464a626e526c6243424462334a7762334a6864476c76626a45744d437347413155454177776b535735305a57776755306459494546306447567a644746306157397549464a6c6347397964434254615764756157356e4d494942496a414e42676b71686b6947397730424151454641414f43415138414d49494243674b434151454171586f74344f5a75706852386e75644672414669614778786b676d612f45732f42412b74626543545552313036414c31454e635741344658334b2b453942424c302f375835726a356e4967582f522f317562686b4b5777396766715047334b654174496463762f75544f3179587635307671615076453143524368767a64532f5a45427151356f56764c54505a3356456963516a6c79744b674e39634c6e7862777475764c554b3765795250664a572f6b7364644f7a50385642426e696f6c596e524344326a724d525a386e424d325a5759776e586e7759654f4148562b5739744f6841496d7752774b462f393579417356776432317279484d4a426347483730714c61675a37547479742b2b714f2f362b4b41584a754b775a716a526c457453457a38675a51654666565967637753666f39366f534d417a56723756304c364853444c526e70623678786d625064714e6f6c3474514944415141426f34476b4d4947684d42384741315564497751594d426141464868446533616d66727a51723335434e2b733166447548415645384d41344741315564447745422f775145417749477744414d42674e5648524d4241663845416a41414d474147413155644877525a4d466377566142546f46474754326830644841364c793930636e567a6447566b63325679646d6c6a5a584d75615735305a577775593239744c324e76626e526c626e517651314a4d4c314e48574339426448526c6333526864476c76626c4a6c6347397964464e705a323570626d64445153356a636d77774451594a4b6f5a496876634e4151454c425141446767474241476349746874634b394956527a347252712b5a4b452b376b35302f4f7855736d57386161764f7a4b62306943783037595139727a69356e553733744d45327947524c7a6853566946732f4c704661396c70514c364a4c316151776d4452373454785947424149693566344935544a6f4343457152487a39316b7047365576796e32744c6d6e49644a62504534765976574c72745858664642535350443441666e372b332f58556767416c63376f4354697a4f666262744f466c59413467354b63596753314a325a41654d51716255645a73655a4363615a5a5a6e363574647165653855585a6c447678302b4e644f304c522b357046792b6a754d307757627535394d767a636d5458626a7369374859367a6435335971354b32343466774648525138654f42304957422b3450664d3746654141705a766c66716c4b4f6c4c635a4c327579566d7a526b79523579573732756f396d65685834344369504a32667365395936655174636645684d506b6d4858493031734e2b4b775062704133392b784f7353746a6850394e3159316132745141566f2b7956674c67563248777337334663306f3377433738715045412b76326152732f4265335a46446744796768632f316667552b37432b50366b62716434706f7962364957384b434a6278664d4a766b6f72644e4f674f5555786e64504845692f74622f5537754c6a4c4f6750413d3d227d300a06082a8648ce3d040302034700304402206734f4d44f9a6cd347db2c1373ba3035d3d31fa21a50ffd9dba22dce7007f33302202aaf8178c59de8fb89cfc1574ffbd264f7b356f1628c92e5a719994fb19aeffc12690a510a460a1f2f636f736d6f732e63727970746f2e736563703235366b312e5075624b657912230a2102511ff07856e34e3716efe44526db0259da8b73841d2ae41030f340c477fc254e12040a02080118a70e12140a0e0a0575736372741205333735303010f093091a40690bf055b9faa32ecefefc34ee4f22bd9a4519b5aec35d5160fc5e2584dfa8c815ebd5c9974780c14640fb274077e04840fd81cc847f7a0ecf8a4a6cae8988c5";

    pub fn parse_tx_basic() {
        let tx_bytes_hex = TX_RAW_SINGLE_WASM_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        assert_eq!(tx.body.unwrap().messages.len(), 1 as usize)
    }

    pub fn parse_tx_multiple_msg() {
        let tx_bytes_hex = TX_RAW_2_WASM_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        assert_eq!(tx.body.unwrap().messages.len(), 2 as usize)
    }

    pub fn parse_tx_multiple_msg_non_wasm() {
        let tx_bytes_hex = TX_RAW_2_WASM_1_BANK_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        assert_eq!(tx.body.unwrap().messages.len(), 3 as usize)
    }

    pub fn test_check_message_not_wasm() {
        let tx_bytes_hex = TX_RAW_2_WASM_1_BANK_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        let msg = tx.body.unwrap().messages[2].clone();

        assert_eq!(super::message_is_wasm(&msg), false)
    }

    pub fn check_message_is_wasm() {
        let tx_bytes_hex = TX_RAW_SINGLE_WASM_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        let msg = tx.body.unwrap().messages[0].clone();

        assert_eq!(super::message_is_wasm(&msg), true)
    }

    pub fn check_message_is_reg() {
        let tx_bytes_hex = TX_RAW_REGISTRATION_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        let msg = tx.body.unwrap().messages[0].clone();
        assert_eq!(super::message_is_reg(&msg), true)
    }

    pub fn check_parse_reg_bytes() {
        let tx_bytes_hex = TX_RAW_REGISTRATION_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = cosmos_proto::registration::v1beta1::msg::RaAuthenticate::parse_from_bytes(
            tx_bytes.as_slice(),
        );

        assert!(tx.is_ok());
    }

    pub fn check_parse_reg_from_tx() {
        let tx_bytes_hex = TX_RAW_REGISTRATION_MSG;

        const EXPECTED_CERTIFICATE: &str = "MIIOazCCDhKgAwIBAgIBATAKBggqhkjOPQQDAjAUMRIwEAYDVQQDDAlTZWNyZXRURUUwHhcNMjMwNTIxMTMzMzIwWhcNMzMwNTIwMTMzMzIwWjAqMSgwJgYDVQQDDB9TZWNyZXQgTmV0d29yayBOb2RlIENlcnRpZmljYXRlMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAEztL7furNh8ta/sUBOoqBrjF9mcWXRyGTtqazm0m7a7t/Q8TzZ27/WcG3yMxI05TqyXJAV7PlZewdwAcrvnoC+KOCDT0wgg05MIINNQYJYIZIAYb4QgENBIINJnsicmVwb3J0IjoiZXlKcFpDSTZJakUyTWpRek1qZ3dPVEUzTnpVME16RTFOamc0TURFNE5URTJORFl3TURneU5EWXpPRGd5TXlJc0luUnBiV1Z6ZEdGdGNDSTZJakl3TWpNdE1EVXRNakZVTVRNNk16TTZNakF1TWpnek1ETTNJaXdpZG1WeWMybHZiaUk2TkN3aVpYQnBaRkJ6WlhWa2IyNTViU0k2SW10QmFpdE5jMWg2Ym1NNWEzTnZTV3h6WVZoWksyWmxiMVpOVW10bFpVWnVaQzloTkRSQ01TdHRNMmhxVENzelpTdDViRmxrVkZNelF5OURjek5DV0dSaFVuSlpXRGR6YzNJeFVHTXJRbU0wVlZKcU9HdHNjaXRuZUdkV04zbFNORmh6Y2k5WVFXWnJNRlpUTkM4MWJqRm9WRVozTWt0a2FXVkNTM2h4VjNaWFdWTm1iM1ZYV1hGdmFqZ3hkMVZ2VTFSNGFtZHFURVJpY0d4cWNFcDJMMEpyYjB0bldtdDZNemcyTUQwaUxDSmhaSFpwYzI5eWVWVlNUQ0k2SW1oMGRIQnpPaTh2YzJWamRYSnBkSGt0WTJWdWRHVnlMbWx1ZEdWc0xtTnZiU0lzSW1Ga2RtbHpiM0o1U1VSeklqcGJJa2xPVkVWTUxWTkJMVEF3TXpNMElpd2lTVTVVUlV3dFUwRXRNREEyTVRVaVhTd2lhWE4yUlc1amJHRjJaVkYxYjNSbFUzUmhkSFZ6SWpvaVUxZGZTRUZTUkVWT1NVNUhYMDVGUlVSRlJDSXNJbWx6ZGtWdVkyeGhkbVZSZFc5MFpVSnZaSGtpT2lKQlowRkNRVVZCVFVGQlFVOUJRVFJCUVVGQlFVRklaemhrWmpCRlNHbHBkVzkwZGs1VFIwWXhaQzgwUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlJYaE5RMEl2SzBGQ2QwRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVSlJRVUZCUVVGQlFVRkJTRUZCUVVGQlFVRkJRVXBQUlcxRWVFTmpXR1ZPTjBScmFsRjNjMEk1UW1ZNVZVTTBaV3RMTDNGbWIwWXhXWGRZTVRWT1drWkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGRFJWaFFUa2xHVUZKV2JHTmpaeXRDT1RCbFZURTBWMVJHU1ZRd1VVWXhkVlZFWW5acFNFVXZPQzl3WjBGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJaMEZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJSR0o2U0RNM1ZUUkxlbFJpYm5SSFpYRXhZelo1VW01eE4yNVBXVkZRYjFjMVFYZGxPRVpHTkRsVldHZEJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVGQlFVRkJRVUZCUVVFaWZRPT0iLCJzaWduYXR1cmUiOiJEa2IxTXNnbzhCWVNrSFBRRm5TTVlKeTlLVU4vRHlsTlBkSERXNDQ3NkJYaDRtTDdMaCtkdjhDRmRyb1NLdzg3NkVUYXhCaFUvT3AreFNVK254NW13Q0tSSkYwVElzVENFdXFvc01OVW1OUHVVRTFSQUhVeEhMT0JCOUNVZ3dkR3FqVXJxcThNSWUrQ2QrbXJ2cVlKbG05ekdRWXBsNXNYeVpva0puTjVhUkY3em1XanhCN25RVFRHMTR4QkozKy9NMTNmWWJ3MU5NbWhtN1pCRHFnQ3F3QzF0NnYxOFBwVVVYK2c1TUNkNHhqZnFPKzBkTHlwdE5GSUlzSVhFRGUzWFloZklMS2sybFIwd29rNEkzVUhheWtjQ0xRZnBFaDBvMW0yTGRWSEcwSERXZW95Y3owWVk4S2xHdXFqVXVTR1N4UnpzbVVPWENoK2wxUXUyZnNuSFE9PSIsInNpZ25pbmdfY2VydCI6Ik1JSUVvVENDQXdtZ0F3SUJBZ0lKQU5FSGRsMHlvN0NXTUEwR0NTcUdTSWIzRFFFQkN3VUFNSDR4Q3pBSkJnTlZCQVlUQWxWVE1Rc3dDUVlEVlFRSURBSkRRVEVVTUJJR0ExVUVCd3dMVTJGdWRHRWdRMnhoY21FeEdqQVlCZ05WQkFvTUVVbHVkR1ZzSUVOdmNuQnZjbUYwYVc5dU1UQXdMZ1lEVlFRRERDZEpiblJsYkNCVFIxZ2dRWFIwWlhOMFlYUnBiMjRnVW1Wd2IzSjBJRk5wWjI1cGJtY2dRMEV3SGhjTk1UWXhNVEl5TURrek5qVTRXaGNOTWpZeE1USXdNRGt6TmpVNFdqQjdNUXN3Q1FZRFZRUUdFd0pWVXpFTE1Ba0dBMVVFQ0F3Q1EwRXhGREFTQmdOVkJBY01DMU5oYm5SaElFTnNZWEpoTVJvd0dBWURWUVFLREJGSmJuUmxiQ0JEYjNKd2IzSmhkR2x2YmpFdE1Dc0dBMVVFQXd3a1NXNTBaV3dnVTBkWUlFRjBkR1Z6ZEdGMGFXOXVJRkpsY0c5eWRDQlRhV2R1YVc1bk1JSUJJakFOQmdrcWhraUc5dzBCQVFFRkFBT0NBUThBTUlJQkNnS0NBUUVBcVhvdDRPWnVwaFI4bnVkRnJBRmlhR3h4a2dtYS9Fcy9CQSt0YmVDVFVSMTA2QUwxRU5jV0E0RlgzSytFOUJCTDAvN1g1cmo1bklnWC9SLzF1YmhrS1d3OWdmcVBHM0tlQXRJZGN2L3VUTzF5WHY1MHZxYVB2RTFDUkNodnpkUy9aRUJxUTVvVnZMVFBaM1ZFaWNRamx5dEtnTjljTG54Ynd0dXZMVUs3ZXlSUGZKVy9rc2RkT3pQOFZCQm5pb2xZblJDRDJqck1SWjhuQk0yWldZd25YbndZZU9BSFYrVzl0T2hBSW13UndLRi85NXlBc1Z3ZDIxcnlITUpCY0dINzBxTGFnWjdUdHl0KytxTy82K0tBWEp1S3dacWpSbEV0U0V6OGdaUWVGZlZZZ2N3U2ZvOTZvU01BelZyN1YwTDZIU0RMUm5wYjZ4eG1iUGRxTm9sNHRRSURBUUFCbzRHa01JR2hNQjhHQTFVZEl3UVlNQmFBRkhoRGUzYW1mcnpRcjM1Q04rczFmRHVIQVZFOE1BNEdBMVVkRHdFQi93UUVBd0lHd0RBTUJnTlZIUk1CQWY4RUFqQUFNR0FHQTFVZEh3UlpNRmN3VmFCVG9GR0dUMmgwZEhBNkx5OTBjblZ6ZEdWa2MyVnlkbWxqWlhNdWFXNTBaV3d1WTI5dEwyTnZiblJsYm5RdlExSk1MMU5IV0M5QmRIUmxjM1JoZEdsdmJsSmxjRzl5ZEZOcFoyNXBibWREUVM1amNtd3dEUVlKS29aSWh2Y05BUUVMQlFBRGdnR0JBR2NJdGh0Y0s5SVZSejRyUnErWktFKzdrNTAvT3hVc21XOGFhdk96S2IwaUN4MDdZUTlyemk1blU3M3RNRTJ5R1JMemhTVmlGcy9McEZhOWxwUUw2SkwxYVF3bURSNzRUeFlHQkFJaTVmNEk1VEpvQ0NFcVJIejkxa3BHNlV2eW4ydExtbklkSmJQRTR2WXZXTHJ0WFhmRkJTU1BENEFmbjcrMy9YVWdnQWxjN29DVGl6T2ZiYnRPRmxZQTRnNUtjWWdTMUoyWkFlTVFxYlVkWnNlWkNjYVpaWm42NXRkcWVlOFVYWmxEdngwK05kTzBMUis1cEZ5K2p1TTB3V2J1NTlNdnpjbVRYYmpzaTdIWTZ6ZDUzWXE1SzI0NGZ3RkhSUThlT0IwSVdCKzRQZk03RmVBQXBadmxmcWxLT2xMY1pMMnV5Vm16Umt5UjV5VzcydW85bWVoWDQ0Q2lQSjJmc2U5WTZlUXRjZkVoTVBrbUhYSTAxc04rS3dQYnBBMzkreE9zU3RqaFA5TjFZMWEydFFBVm8reVZnTGdWMkh3czczRmMwbzN3Qzc4cVBFQSt2MmFScy9CZTNaRkRnRHlnaGMvMWZnVSs3QytQNmticWQ0cG95YjZJVzhLQ0pieGZNSnZrb3JkTk9nT1VVeG5kUEhFaS90Yi9VN3VMakxPZ1BBPT0ifTAKBggqhkjOPQQDAgNHADBEAiBnNPTUT5ps00fbLBNzujA109MfohpQ/9nboi3OcAfzMwIgKq+BeMWd6PuJz8FXT/vSZPezVvFijJLlpxmZT7Ga7/w=";

        let cert_decoded = base64::decode(&EXPECTED_CERTIFICATE).unwrap();

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        let msg = tx.body.unwrap().messages[0].clone();

        let tx =
            cosmos_proto::registration::v1beta1::msg::RaAuthenticate::parse_from_bytes(&msg.value)
                .unwrap();

        assert_eq!(tx.certificate, cert_decoded)
    }

    pub fn parse_tx_multisig() {
        let tx_bytes_hex = TX_RAW_MULTISIG_1_WASM_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        assert_eq!(tx.clone().body.unwrap().messages.len(), 1 as usize);

        let msg = tx.clone().body.unwrap().messages[0].clone();

        assert_eq!(super::message_is_wasm(&msg), true)
    }

    pub fn test_wasm_msg_tracker() {
        let tx_bytes_hex = TX_RAW_SINGLE_WASM_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        let ref_tx = tx.clone();

        super::VERIFIED_MESSAGES
            .lock()
            .unwrap()
            .append_msg_from_tx(tx);

        assert_eq!(
            super::VERIFIED_MESSAGES.lock().unwrap().remaining(),
            1 as usize
        );
        assert_eq!(
            super::VERIFIED_MESSAGES.lock().unwrap().get_next().unwrap(),
            ref_tx.body.unwrap().messages[0].value
        );
        assert_eq!(
            super::VERIFIED_MESSAGES.lock().unwrap().remaining(),
            0 as usize
        );
    }

    pub fn test_wasm_msg_tracker_multiple_msgs() {
        let tx_bytes_hex = TX_RAW_2_WASM_1_BANK_MSG;

        let tx_bytes = hex::decode(tx_bytes_hex).unwrap();

        let tx = protoTx::tx::Tx::parse_from_bytes(tx_bytes.as_slice()).unwrap();

        let ref_tx = tx.clone();
        let ref_msgs = ref_tx.body.unwrap().messages;
        super::VERIFIED_MESSAGES
            .lock()
            .unwrap()
            .append_msg_from_tx(tx);

        assert_eq!(
            super::VERIFIED_MESSAGES.lock().unwrap().remaining(),
            2 as usize
        );
        assert_eq!(
            &super::VERIFIED_MESSAGES.lock().unwrap().get_next().unwrap(),
            &ref_msgs[0].value
        );
        assert_eq!(
            super::VERIFIED_MESSAGES.lock().unwrap().remaining(),
            1 as usize
        );
        assert_eq!(
            &super::VERIFIED_MESSAGES.lock().unwrap().get_next().unwrap(),
            &ref_msgs[1].value
        );
        assert_eq!(
            super::VERIFIED_MESSAGES.lock().unwrap().remaining(),
            0 as usize
        );
    }
    //
}
