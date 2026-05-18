# GovChain Simulation Steps

This file outlines a minimal end-to-end flow that can be simulated with the current smart contract interface.

## Setup
1. Deploy a token contract for fiat (SAC or custom token) and note its address.
2. Deploy the GovChain contract and note its address.
3. Call `initialize(kppn_admin, fiat_token_address, registry_contract)`.
   - For internal vendor registry simulation, set `registry_contract` to the GovChain contract address.
4. (Optional) Fund the treasury with `mint_fiat(kppn_admin, kppn_admin, amount)` if the token allows admin minting.

## Role Registration (Multi-role)
Call `tetapkan_peran_instansi` with lists of addresses:
- `kpa`: one or more KPA addresses
- `ppk`: one or more PPK addresses
- `treasurer`: one or more Treasurer addresses

## Vendor Registry (Simulation)
If using internal registry:
- `registrasi_vendor(kppn_admin, vendor_address, true)`

## Budget Allocation and UP Limit
1. `alokasikan_pagu_dipa(kppn_admin, instansi, pagu, waktu_kedaluwarsa)`
2. `tetapkan_batas_up(kpa, instansi, batas_up)`

## LS Flow (Direct Payment)
1. `ajukan_spm(ppk, payload)` with `spm_type = Ls` and `address_penerima = vendor_address`
2. `otorisasi_kpa_spm(kpa, spm_id)`
3. `eksekusi_pencairan_sp2d(kppn_admin, spm_id)`

## UP Flow (Operational Cash)
1. `ajukan_spm(ppk, payload)` with `spm_type = Up` and `address_penerima = treasurer_address`
2. `otorisasi_kpa_spm(kpa, spm_id)`
3. `eksekusi_pencairan_sp2d(kppn_admin, spm_id)`

## UP Usage and GUP Replenishment
1. `lapor_penggunaan_up(treasurer, instansi, nominal_terpakai)`
2. `ajukan_gup(treasurer, payload)` with `spm_type = Gup` and `address_penerima = treasurer_address`
3. `otorisasi_kpa_spm(kpa, spm_id)`
4. `eksekusi_pencairan_sp2d(kppn_admin, spm_id)`

## Audit and Queries
- `lihat_dipa(instansi)`
- `lihat_spm(spm_id)`
- `lihat_peran_instansi(instansi)`
- `lihat_up_state(instansi)`

Notes:
- All monetary values are `i128` in the token's smallest unit.
- If you use an external registry contract, it must implement `is_vendor_valid(Address) -> bool`.
- UP limit enforcement is based on `UpState.batas`, and UP/GUP reservations are tracked in `UpState.reserved`.
