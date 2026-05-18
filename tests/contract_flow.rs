use soroban_sdk::{token, Address, BytesN, Env, Vec};
use soroban_sdk::testutils::{Address as _, BytesN as _};

use govchain::{
    DokumenSpmPayload, GovChainContract, GovChainContractClient, SpmType, StatusPencairan,
};

struct TestCtx {
    env: Env,
    client: GovChainContractClient,
    token: token::Client,
    admin: Address,
    instansi: Address,
    kpa: Address,
    ppk: Address,
    treasurer: Address,
    vendor: Address,
}

fn setup() -> TestCtx {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::random(&env);
    let instansi = Address::random(&env);
    let kpa = Address::random(&env);
    let ppk = Address::random(&env);
    let treasurer = Address::random(&env);
    let vendor = Address::random(&env);

    let token_id = env.register_stellar_asset_contract(admin.clone());
    let token = token::Client::new(&env, &token_id);

    let contract_id = env.register_contract(None, GovChainContract);
    let client = GovChainContractClient::new(&env, &contract_id);

    client.initialize(&admin, &token_id, &contract_id);
    client.mint_fiat(&admin, &admin, &1_000_000);

    let kpa_list = Vec::from_array(&env, [kpa.clone()]);
    let ppk_list = Vec::from_array(&env, [ppk.clone()]);
    let treasurer_list = Vec::from_array(&env, [treasurer.clone()]);
    client.tetapkan_peran_instansi(&admin, &instansi, &kpa_list, &ppk_list, &treasurer_list);

    client.registrasi_vendor(&admin, &vendor, &true);
    client.alokasikan_pagu_dipa(&admin, &instansi, &500_000, &0);
    client.tetapkan_batas_up(&kpa, &instansi, &100_000);

    TestCtx {
        env,
        client,
        token,
        admin,
        instansi,
        kpa,
        ppk,
        treasurer,
        vendor,
    }
}

#[test]
fn ls_flow_end_to_end() {
    let ctx = setup();
    let env = &ctx.env;

    let spm_id = BytesN::random(env);
    let payload = DokumenSpmPayload {
        id_spm: spm_id.clone(),
        spm_type: SpmType::Ls,
        address_instansi: ctx.instansi.clone(),
        address_penerima: ctx.vendor.clone(),
        nominal_pencairan: 50_000,
        hash_kontrak_vendor: BytesN::random(env),
    };

    ctx.client.ajukan_spm(&ctx.ppk, &payload);
    ctx.client.otorisasi_kpa_spm(&ctx.kpa, &spm_id);
    ctx.client.eksekusi_pencairan_sp2d(&ctx.admin, &spm_id);

    let spm = ctx.client.lihat_spm(&spm_id);
    assert_eq!(spm.status, StatusPencairan::Selesai);

    let vendor_balance = ctx.token.balance(&ctx.vendor);
    assert_eq!(vendor_balance, 50_000);

    let dipa = ctx.client.lihat_dipa(&ctx.instansi);
    assert_eq!(dipa.sisa_saldo_aktif, 450_000);
}

#[test]
fn up_and_gup_flow_end_to_end() {
    let ctx = setup();
    let env = &ctx.env;

    let spm_id_up = BytesN::random(env);
    let payload_up = DokumenSpmPayload {
        id_spm: spm_id_up.clone(),
        spm_type: SpmType::Up,
        address_instansi: ctx.instansi.clone(),
        address_penerima: ctx.treasurer.clone(),
        nominal_pencairan: 25_000,
        hash_kontrak_vendor: BytesN::random(env),
    };

    ctx.client.ajukan_spm(&ctx.ppk, &payload_up);
    ctx.client.otorisasi_kpa_spm(&ctx.kpa, &spm_id_up);
    ctx.client.eksekusi_pencairan_sp2d(&ctx.admin, &spm_id_up);

    let up_state = ctx.client.lihat_up_state(&ctx.instansi);
    assert_eq!(up_state.saldo, 25_000);
    assert_eq!(up_state.reserved, 0);

    ctx.client.lapor_penggunaan_up(&ctx.treasurer, &ctx.instansi, &10_000);
    let up_state = ctx.client.lihat_up_state(&ctx.instansi);
    assert_eq!(up_state.saldo, 15_000);
    assert_eq!(up_state.terpakai, 10_000);

    let spm_id_gup = BytesN::random(env);
    let payload_gup = DokumenSpmPayload {
        id_spm: spm_id_gup.clone(),
        spm_type: SpmType::Gup,
        address_instansi: ctx.instansi.clone(),
        address_penerima: ctx.treasurer.clone(),
        nominal_pencairan: 10_000,
        hash_kontrak_vendor: BytesN::random(env),
    };

    ctx.client.ajukan_gup(&ctx.treasurer, &payload_gup);
    ctx.client.otorisasi_kpa_spm(&ctx.kpa, &spm_id_gup);
    ctx.client.eksekusi_pencairan_sp2d(&ctx.admin, &spm_id_gup);

    let up_state = ctx.client.lihat_up_state(&ctx.instansi);
    assert_eq!(up_state.saldo, 25_000);
    assert_eq!(up_state.terpakai, 0);

    let dipa = ctx.client.lihat_dipa(&ctx.instansi);
    assert_eq!(dipa.sisa_saldo_aktif, 475_000);
}
