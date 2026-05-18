use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, token, Address,
    BytesN, Env, IntoVal, Symbol, Val, Vec,
};

// ==========================================
// DATA STRUCTURES
// ==========================================

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum SpmType {
    Ls, // Direct Payment to Third Party
    Up, // Operational Petty Cash for Satker
    Gup, // Replenishment of UP after reconciliation
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StatusPencairan {
    DrafPPK,     // Initiated, no balance impact
    MenungguKpa, // Basic verification done, awaiting final commit
    SiapCair,    // Fully authorized, awaiting KPPN execution
    Selesai,     // SP2D executed, assets transferred
    Retur,       // Automated system rejection
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlokasiDipaSatker {
    pub total_pagu_tahunan: i128,
    pub realisasi_pengeluaran: i128,
    pub sisa_saldo_aktif: i128,
    pub waktu_kedaluwarsa: u32, // Based on ledger/block boundaries
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DokumenSpmPayload {
    pub id_spm: BytesN<32>,
    pub spm_type: SpmType,
    pub address_instansi: Address,
    pub address_penerima: Address,
    pub nominal_pencairan: i128,
    pub hash_kontrak_vendor: BytesN<32>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpmRecord {
    pub payload: DokumenSpmPayload,
    pub status: StatusPencairan,
    pub ppk: Address,
    pub created_ledger: u32,
    pub approved_ledger: u32,
    pub executed_ledger: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InstansiRoles {
    pub kpa: Vec<Address>,
    pub ppk: Vec<Address>,
    pub treasurer: Vec<Address>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpState {
    pub batas: i128,
    pub saldo: i128,
    pub terpakai: i128,
    pub reserved: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    KppnAdmin,
    FiatToken,
    Registry,
    Dipa(Address),
    Roles(Address),
    UpState(Address),
    Spm(BytesN<32>),
    Vendor(Address),
}

// ==========================================
// ERROR HANDLING
// ==========================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GovChainError {
    DefisitPaguDipa = 1001,             // Allocation deficit; claim exceeds DIPA ceiling
    PenolakanRegistrasiSupplier = 1002, // Vendor cross-verification failed in Registry
    OtorisasiPemanggilKorupsi = 1003,   // Authorization hijacking; Caller lacks KPA/PPK rights
    KedaluwarsaPeriodeAnggaran = 1004,  // Budget period expired; blocked by ledger boundary
    SudahDiinisialisasi = 1005,
    DataTidakDitemukan = 1006,
    StatusTidakValid = 1007,
    NominalTidakValid = 1008,
    PeranTidakTerdaftar = 1009,
    PenerimaTidakSesuai = 1010,
    BatasUpBelumDiatur = 1011,
    SaldoUpTidakCukup = 1012,
    GupMelebihiPemakaian = 1013,
    JenisSpmTidakValid = 1014,
    MelebihiBatasUp = 1015,
}

// ==========================================
// SMART CONTRACT INTERFACE
// ==========================================

pub trait GovChainTrait {
    /// 1. KPPN Executive Administration (Initialization & Allocation)
    /// Once-only initialization mapping Super Admin and Registry endpoints
    fn initialize(
        env: Env, 
        kppn_admin: Address, 
        fiat_token_address: Address, 
        registry_contract: Address
    );

    /// Pumps indicative liquidity into a working unit (Requires Admin Auth)
    fn alokasikan_pagu_dipa(
        env: Env, 
        admin: Address, 
        target_instansi: Address, 
        nominal_pagu: i128, 
        waktu_kedaluwarsa: u32
    );

    /// Registry peran instansi untuk simulasi (KPA, PPK, Treasurer)
    fn tetapkan_peran_instansi(
        env: Env,
        admin: Address,
        instansi: Address,
        kpa: Vec<Address>,
        ppk: Vec<Address>,
        treasurer: Vec<Address>,
    );

    /// KPA menetapkan batas UP maksimum
    fn tetapkan_batas_up(
        env: Env,
        kpa: Address,
        instansi: Address,
        batas_up: i128,
    );

    /// 2. Submission Module (Decentralized SPM Phase)
    /// PPK converts physical payment order into a blockchain proposal
    fn ajukan_spm(
        env: Env, 
        inisiator_ppk: Address, 
        payload_pencairan: DokumenSpmPayload
    ) -> BytesN<32>;

    /// Treasurer memicu GUP (replenishment) setelah rekonsiliasi pemakaian
    fn ajukan_gup(
        env: Env,
        treasurer: Address,
        payload_pencairan: DokumenSpmPayload,
    ) -> BytesN<32>;

    /// KPA multi-sig finalization. Checks DIPA balance and updates status to `SiapCair`
    fn otorisasi_kpa_spm(
        env: Env, 
        otorisator_kpa: Address, 
        spm_id: BytesN<32>
    );

    /// 3. Disbursement Infrastructure (SP2D Execution)
    /// Final execution. Validates `SiapCair` status and transfers assets atomically to the vendor
    fn eksekusi_pencairan_sp2d(
        env: Env, 
        kasir_negara: Address, 
        spm_id: BytesN<32>
    );

    /// Laporan pemakaian UP oleh Treasurer
    fn lapor_penggunaan_up(
        env: Env,
        treasurer: Address,
        instansi: Address,
        nominal_terpakai: i128,
    );

    /// Mint fiat untuk simulasi (opsional)
    fn mint_fiat(
        env: Env,
        admin: Address,
        tujuan: Address,
        nominal: i128,
    );

    /// Registry vendor sederhana untuk simulasi
    fn registrasi_vendor(
        env: Env,
        admin: Address,
        vendor: Address,
        status_valid: bool,
    );

    /// Endpoint kompatibel untuk kontrak registry
    fn is_vendor_valid(env: Env, vendor: Address) -> bool;

    /// Read-only endpoints untuk audit/simulasi
    fn lihat_dipa(env: Env, instansi: Address) -> AlokasiDipaSatker;
    fn lihat_spm(env: Env, spm_id: BytesN<32>) -> SpmRecord;
    fn lihat_peran_instansi(env: Env, instansi: Address) -> InstansiRoles;
    fn lihat_up_state(env: Env, instansi: Address) -> UpState;
}

#[contract]
pub struct GovChainContract;

const DEFAULT_TTL_LEDGERS: u32 = 518_400;
const DEFAULT_TTL_THRESHOLD: u32 = 172_800;

enum RoleKind {
    Kpa,
    Ppk,
    Treasurer,
}

fn read_instance_address(env: &Env, key: DataKey) -> Address {
    match env.storage().instance().get(&key) {
        Some(addr) => addr,
        None => panic_with_error!(env, GovChainError::DataTidakDitemukan),
    }
}

fn read_admin(env: &Env) -> Address {
    read_instance_address(env, DataKey::KppnAdmin)
}

fn read_token(env: &Env) -> Address {
    read_instance_address(env, DataKey::FiatToken)
}

fn read_registry(env: &Env) -> Address {
    read_instance_address(env, DataKey::Registry)
}

fn read_roles(env: &Env, instansi: &Address) -> InstansiRoles {
    let key = DataKey::Roles(instansi.clone());
    match env.storage().persistent().get(&key) {
        Some(roles) => roles,
        None => panic_with_error!(env, GovChainError::PeranTidakTerdaftar),
    }
}

fn read_up_state(env: &Env, instansi: &Address) -> UpState {
    let key = DataKey::UpState(instansi.clone());
    match env.storage().persistent().get(&key) {
        Some(state) => state,
        None => UpState {
            batas: 0,
            saldo: 0,
            terpakai: 0,
            reserved: 0,
        },
    }
}

fn write_up_state(env: &Env, instansi: &Address, state: &UpState) {
    let key = DataKey::UpState(instansi.clone());
    env.storage().persistent().set(&key, state);
    extend_persistent_ttl(env, &key);
}

fn address_in_list(list: &Vec<Address>, target: &Address) -> bool {
    let mut i: u32 = 0;
    while i < list.len() {
        if let Some(item) = list.get(i) {
            if &item == target {
                return true;
            }
        }
        i += 1;
    }
    false
}

fn ensure_role(roles: &InstansiRoles, kind: RoleKind, actor: &Address) -> bool {
    match kind {
        RoleKind::Kpa => address_in_list(&roles.kpa, actor),
        RoleKind::Ppk => address_in_list(&roles.ppk, actor),
        RoleKind::Treasurer => address_in_list(&roles.treasurer, actor),
    }
}

fn read_vendor_status(env: &Env, vendor: &Address) -> bool {
    let key = DataKey::Vendor(vendor.clone());
    match env.storage().persistent().get(&key) {
        Some(status) => status,
        None => false,
    }
}

fn extend_persistent_ttl(env: &Env, key: &DataKey) {
    env.storage()
        .persistent()
        .extend_ttl(key, DEFAULT_TTL_THRESHOLD, DEFAULT_TTL_LEDGERS);
}

fn require_admin_auth(env: &Env, admin: &Address) {
    let stored_admin = read_admin(env);
    if &stored_admin != admin {
        panic_with_error!(env, GovChainError::OtorisasiPemanggilKorupsi);
    }
    admin.require_auth();
}

fn ensure_nominal_positive(env: &Env, nominal: i128) {
    if nominal <= 0 {
        panic_with_error!(env, GovChainError::NominalTidakValid);
    }
}

fn ensure_dipa_active(env: &Env, dipa: &AlokasiDipaSatker) {
    let now = env.ledger().sequence();
    if dipa.waktu_kedaluwarsa != 0 && now > dipa.waktu_kedaluwarsa {
        panic_with_error!(env, GovChainError::KedaluwarsaPeriodeAnggaran);
    }
}

fn ensure_up_limit_set(env: &Env, state: &UpState) {
    if state.batas <= 0 {
        panic_with_error!(env, GovChainError::BatasUpBelumDiatur);
    }
}

fn check_vendor_registry(env: &Env, vendor: &Address) {
    let registry = read_registry(env);
    if registry == env.current_contract_address() {
        if !read_vendor_status(env, vendor) {
            panic_with_error!(env, GovChainError::PenolakanRegistrasiSupplier);
        }
        return;
    }

    let mut args = Vec::<Val>::new(env);
    args.push_back(vendor.clone().into_val(env));
    let valid: bool = env.invoke_contract(&registry, &Symbol::new(env, "is_vendor_valid"), args);
    if !valid {
        panic_with_error!(env, GovChainError::PenolakanRegistrasiSupplier);
    }
}

#[contractimpl]
impl GovChainTrait for GovChainContract {
    fn initialize(env: Env, kppn_admin: Address, fiat_token_address: Address, registry_contract: Address) {
        if env.storage().instance().has(&DataKey::KppnAdmin) {
            panic_with_error!(&env, GovChainError::SudahDiinisialisasi);
        }

        kppn_admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::KppnAdmin, &kppn_admin);
        env.storage()
            .instance()
            .set(&DataKey::FiatToken, &fiat_token_address);
        env.storage()
            .instance()
            .set(&DataKey::Registry, &registry_contract);
    }

    fn alokasikan_pagu_dipa(env: Env, admin: Address, target_instansi: Address, nominal_pagu: i128, waktu_kedaluwarsa: u32) {
        require_admin_auth(&env, &admin);
        ensure_nominal_positive(&env, nominal_pagu);

        let key = DataKey::Dipa(target_instansi.clone());
        let mut dipa = match env.storage().persistent().get(&key) {
            Some(existing) => existing,
            None => AlokasiDipaSatker {
                total_pagu_tahunan: 0,
                realisasi_pengeluaran: 0,
                sisa_saldo_aktif: 0,
                waktu_kedaluwarsa,
            },
        };

        dipa.total_pagu_tahunan += nominal_pagu;
        dipa.sisa_saldo_aktif += nominal_pagu;
        if dipa.waktu_kedaluwarsa < waktu_kedaluwarsa {
            dipa.waktu_kedaluwarsa = waktu_kedaluwarsa;
        }

        env.storage().persistent().set(&key, &dipa);
        extend_persistent_ttl(&env, &key);
    }

    fn tetapkan_peran_instansi(
        env: Env,
        admin: Address,
        instansi: Address,
        kpa: Vec<Address>,
        ppk: Vec<Address>,
        treasurer: Vec<Address>,
    ) {
        require_admin_auth(&env, &admin);

        let roles = InstansiRoles {
            kpa,
            ppk,
            treasurer,
        };
        let key = DataKey::Roles(instansi);
        env.storage().persistent().set(&key, &roles);
        extend_persistent_ttl(&env, &key);
        env.events().publish((Symbol::new(&env, "roles"),), true);
    }

    fn tetapkan_batas_up(env: Env, kpa: Address, instansi: Address, batas_up: i128) {
        kpa.require_auth();
        ensure_nominal_positive(&env, batas_up);

        let roles = read_roles(&env, &instansi);
        if !ensure_role(&roles, RoleKind::Kpa, &kpa) {
            panic_with_error!(&env, GovChainError::OtorisasiPemanggilKorupsi);
        }

        let mut state = read_up_state(&env, &instansi);
        state.batas = batas_up;
        write_up_state(&env, &instansi, &state);
        env.events().publish((Symbol::new(&env, "up_limit"),), batas_up);
    }

    fn ajukan_spm(env: Env, inisiator_ppk: Address, payload_pencairan: DokumenSpmPayload) -> BytesN<32> {
        inisiator_ppk.require_auth();
        ensure_nominal_positive(&env, payload_pencairan.nominal_pencairan);

        if payload_pencairan.spm_type == SpmType::Gup {
            panic_with_error!(&env, GovChainError::JenisSpmTidakValid);
        }

        let roles = read_roles(&env, &payload_pencairan.address_instansi);
        if !ensure_role(&roles, RoleKind::Ppk, &inisiator_ppk) {
            panic_with_error!(&env, GovChainError::OtorisasiPemanggilKorupsi);
        }

        let dipa_key = DataKey::Dipa(payload_pencairan.address_instansi.clone());
        let dipa: AlokasiDipaSatker = match env.storage().persistent().get(&dipa_key) {
            Some(existing) => existing,
            None => panic_with_error!(&env, GovChainError::DataTidakDitemukan),
        };
        ensure_dipa_active(&env, &dipa);

        if payload_pencairan.spm_type == SpmType::Ls {
            check_vendor_registry(&env, &payload_pencairan.address_penerima);
        } else if !address_in_list(&roles.treasurer, &payload_pencairan.address_penerima) {
            panic_with_error!(&env, GovChainError::PenerimaTidakSesuai);
        }

        let spm_key = DataKey::Spm(payload_pencairan.id_spm.clone());
        if env.storage().persistent().has(&spm_key) {
            panic_with_error!(&env, GovChainError::StatusTidakValid);
        }

        let record = SpmRecord {
            payload: payload_pencairan.clone(),
            status: StatusPencairan::MenungguKpa,
            ppk: inisiator_ppk,
            created_ledger: env.ledger().sequence(),
            approved_ledger: 0,
            executed_ledger: 0,
        };

        env.storage().persistent().set(&spm_key, &record);
        extend_persistent_ttl(&env, &spm_key);
        env.events().publish(
            (Symbol::new(&env, "spm"), payload_pencairan.id_spm.clone()),
            record.status,
        );

        payload_pencairan.id_spm
    }

    fn ajukan_gup(env: Env, treasurer: Address, payload_pencairan: DokumenSpmPayload) -> BytesN<32> {
        treasurer.require_auth();
        ensure_nominal_positive(&env, payload_pencairan.nominal_pencairan);

        if payload_pencairan.spm_type != SpmType::Gup {
            panic_with_error!(&env, GovChainError::JenisSpmTidakValid);
        }

        let roles = read_roles(&env, &payload_pencairan.address_instansi);
        if !ensure_role(&roles, RoleKind::Treasurer, &treasurer) {
            panic_with_error!(&env, GovChainError::OtorisasiPemanggilKorupsi);
        }

        if &payload_pencairan.address_penerima != &treasurer {
            panic_with_error!(&env, GovChainError::PenerimaTidakSesuai);
        }

        let dipa_key = DataKey::Dipa(payload_pencairan.address_instansi.clone());
        let dipa: AlokasiDipaSatker = match env.storage().persistent().get(&dipa_key) {
            Some(existing) => existing,
            None => panic_with_error!(&env, GovChainError::DataTidakDitemukan),
        };
        ensure_dipa_active(&env, &dipa);

        let mut up_state = read_up_state(&env, &payload_pencairan.address_instansi);
        ensure_up_limit_set(&env, &up_state);
        if up_state.terpakai < payload_pencairan.nominal_pencairan {
            panic_with_error!(&env, GovChainError::GupMelebihiPemakaian);
        }

        let spm_key = DataKey::Spm(payload_pencairan.id_spm.clone());
        if env.storage().persistent().has(&spm_key) {
            panic_with_error!(&env, GovChainError::StatusTidakValid);
        }

        let record = SpmRecord {
            payload: payload_pencairan.clone(),
            status: StatusPencairan::MenungguKpa,
            ppk: treasurer,
            created_ledger: env.ledger().sequence(),
            approved_ledger: 0,
            executed_ledger: 0,
        };

        env.storage().persistent().set(&spm_key, &record);
        extend_persistent_ttl(&env, &spm_key);
        env.events().publish(
            (Symbol::new(&env, "spm"), payload_pencairan.id_spm.clone()),
            record.status,
        );

        payload_pencairan.id_spm
    }

    fn otorisasi_kpa_spm(env: Env, otorisator_kpa: Address, spm_id: BytesN<32>) {
        otorisator_kpa.require_auth();

        let spm_key = DataKey::Spm(spm_id.clone());
        let mut record: SpmRecord = match env.storage().persistent().get(&spm_key) {
            Some(existing) => existing,
            None => panic_with_error!(&env, GovChainError::DataTidakDitemukan),
        };

        if record.status != StatusPencairan::MenungguKpa && record.status != StatusPencairan::DrafPPK {
            panic_with_error!(&env, GovChainError::StatusTidakValid);
        }

        let roles = read_roles(&env, &record.payload.address_instansi);
        if !ensure_role(&roles, RoleKind::Kpa, &otorisator_kpa) {
            panic_with_error!(&env, GovChainError::OtorisasiPemanggilKorupsi);
        }

        let dipa_key = DataKey::Dipa(record.payload.address_instansi.clone());
        let mut dipa: AlokasiDipaSatker = match env.storage().persistent().get(&dipa_key) {
            Some(existing) => existing,
            None => panic_with_error!(&env, GovChainError::DataTidakDitemukan),
        };
        ensure_dipa_active(&env, &dipa);

        match record.payload.spm_type {
            SpmType::Ls => {
                check_vendor_registry(&env, &record.payload.address_penerima);
                if dipa.sisa_saldo_aktif < record.payload.nominal_pencairan {
                    panic_with_error!(&env, GovChainError::DefisitPaguDipa);
                }
                dipa.sisa_saldo_aktif -= record.payload.nominal_pencairan;
                dipa.realisasi_pengeluaran += record.payload.nominal_pencairan;
                env.storage().persistent().set(&dipa_key, &dipa);
                extend_persistent_ttl(&env, &dipa_key);
            }
            SpmType::Up => {
                if !address_in_list(&roles.treasurer, &record.payload.address_penerima) {
                    panic_with_error!(&env, GovChainError::PenerimaTidakSesuai);
                }
                if dipa.sisa_saldo_aktif < record.payload.nominal_pencairan {
                    panic_with_error!(&env, GovChainError::DefisitPaguDipa);
                }

                let mut up_state = read_up_state(&env, &record.payload.address_instansi);
                ensure_up_limit_set(&env, &up_state);
                if up_state.saldo + up_state.reserved + record.payload.nominal_pencairan > up_state.batas {
                    panic_with_error!(&env, GovChainError::MelebihiBatasUp);
                }
                up_state.reserved += record.payload.nominal_pencairan;
                write_up_state(&env, &record.payload.address_instansi, &up_state);

                dipa.sisa_saldo_aktif -= record.payload.nominal_pencairan;
                dipa.realisasi_pengeluaran += record.payload.nominal_pencairan;
                env.storage().persistent().set(&dipa_key, &dipa);
                extend_persistent_ttl(&env, &dipa_key);
            }
            SpmType::Gup => {
                if !address_in_list(&roles.treasurer, &record.payload.address_penerima) {
                    panic_with_error!(&env, GovChainError::PenerimaTidakSesuai);
                }

                let mut up_state = read_up_state(&env, &record.payload.address_instansi);
                ensure_up_limit_set(&env, &up_state);
                if up_state.terpakai < record.payload.nominal_pencairan {
                    panic_with_error!(&env, GovChainError::GupMelebihiPemakaian);
                }
                if up_state.saldo + up_state.reserved + record.payload.nominal_pencairan > up_state.batas {
                    panic_with_error!(&env, GovChainError::MelebihiBatasUp);
                }

                up_state.terpakai -= record.payload.nominal_pencairan;
                up_state.reserved += record.payload.nominal_pencairan;
                write_up_state(&env, &record.payload.address_instansi, &up_state);
            }
        }

        record.status = StatusPencairan::SiapCair;
        record.approved_ledger = env.ledger().sequence();
        env.storage().persistent().set(&spm_key, &record);
        extend_persistent_ttl(&env, &spm_key);
        env.events().publish(
            (Symbol::new(&env, "spm"), spm_id),
            record.status,
        );
    }

    fn eksekusi_pencairan_sp2d(env: Env, kasir_negara: Address, spm_id: BytesN<32>) {
        require_admin_auth(&env, &kasir_negara);

        let spm_key = DataKey::Spm(spm_id.clone());
        let mut record: SpmRecord = match env.storage().persistent().get(&spm_key) {
            Some(existing) => existing,
            None => panic_with_error!(&env, GovChainError::DataTidakDitemukan),
        };

        if record.status != StatusPencairan::SiapCair {
            panic_with_error!(&env, GovChainError::StatusTidakValid);
        }

        let roles = read_roles(&env, &record.payload.address_instansi);
        match record.payload.spm_type {
            SpmType::Ls => {
                check_vendor_registry(&env, &record.payload.address_penerima);
            }
            SpmType::Up | SpmType::Gup => {
                if !address_in_list(&roles.treasurer, &record.payload.address_penerima) {
                    panic_with_error!(&env, GovChainError::PenerimaTidakSesuai);
                }
                record.payload.address_penerima.require_auth();
            }
        }

        let token_address = read_token(&env);
        let client = token::Client::new(&env, &token_address);
        client.transfer(
            &kasir_negara,
            &record.payload.address_penerima,
            &record.payload.nominal_pencairan,
        );

        if record.payload.spm_type == SpmType::Up || record.payload.spm_type == SpmType::Gup {
            let mut up_state = read_up_state(&env, &record.payload.address_instansi);
            if up_state.reserved < record.payload.nominal_pencairan {
                panic_with_error!(&env, GovChainError::SaldoUpTidakCukup);
            }
            up_state.reserved -= record.payload.nominal_pencairan;
            up_state.saldo += record.payload.nominal_pencairan;
            write_up_state(&env, &record.payload.address_instansi, &up_state);
        }

        record.status = StatusPencairan::Selesai;
        record.executed_ledger = env.ledger().sequence();
        env.storage().persistent().set(&spm_key, &record);
        extend_persistent_ttl(&env, &spm_key);
        env.events().publish(
            (Symbol::new(&env, "spm"), spm_id),
            record.status,
        );
    }

    fn lapor_penggunaan_up(env: Env, treasurer: Address, instansi: Address, nominal_terpakai: i128) {
        treasurer.require_auth();
        ensure_nominal_positive(&env, nominal_terpakai);

        let roles = read_roles(&env, &instansi);
        if !ensure_role(&roles, RoleKind::Treasurer, &treasurer) {
            panic_with_error!(&env, GovChainError::OtorisasiPemanggilKorupsi);
        }

        let mut up_state = read_up_state(&env, &instansi);
        ensure_up_limit_set(&env, &up_state);
        if up_state.saldo < nominal_terpakai {
            panic_with_error!(&env, GovChainError::SaldoUpTidakCukup);
        }

        up_state.saldo -= nominal_terpakai;
        up_state.terpakai += nominal_terpakai;
        write_up_state(&env, &instansi, &up_state);
        env.events().publish((Symbol::new(&env, "up_use"),), nominal_terpakai);
    }

    fn mint_fiat(env: Env, admin: Address, tujuan: Address, nominal: i128) {
        require_admin_auth(&env, &admin);
        ensure_nominal_positive(&env, nominal);

        let token_address = read_token(&env);
        let mut args = Vec::<Val>::new(&env);
        args.push_back(admin.clone().into_val(&env));
        args.push_back(tujuan.into_val(&env));
        args.push_back(nominal.into_val(&env));
        let _: () = env.invoke_contract(&token_address, &Symbol::new(&env, "mint"), args);
        env.events().publish((Symbol::new(&env, "mint"),), nominal);
    }

    fn registrasi_vendor(env: Env, admin: Address, vendor: Address, status_valid: bool) {
        require_admin_auth(&env, &admin);
        let key = DataKey::Vendor(vendor);
        env.storage().persistent().set(&key, &status_valid);
        extend_persistent_ttl(&env, &key);
        env.events().publish((Symbol::new(&env, "vendor"),), status_valid);
    }

    fn is_vendor_valid(env: Env, vendor: Address) -> bool {
        read_vendor_status(&env, &vendor)
    }

    fn lihat_dipa(env: Env, instansi: Address) -> AlokasiDipaSatker {
        let key = DataKey::Dipa(instansi);
        match env.storage().persistent().get(&key) {
            Some(dipa) => dipa,
            None => panic_with_error!(&env, GovChainError::DataTidakDitemukan),
        }
    }

    fn lihat_spm(env: Env, spm_id: BytesN<32>) -> SpmRecord {
        let key = DataKey::Spm(spm_id);
        match env.storage().persistent().get(&key) {
            Some(record) => record,
            None => panic_with_error!(&env, GovChainError::DataTidakDitemukan),
        }
    }

    fn lihat_peran_instansi(env: Env, instansi: Address) -> InstansiRoles {
        read_roles(&env, &instansi)
    }

    fn lihat_up_state(env: Env, instansi: Address) -> UpState {
        read_up_state(&env, &instansi)
    }
}