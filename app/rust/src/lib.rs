#![no_std]
#![no_builtins]
#![allow(dead_code, unused_imports)]

mod bolos;
mod constants;

extern crate core;

use jubjub::{AffineNielsPoint, AffinePoint, ExtendedPoint, Fq, Fr};

use blake2s_simd::{blake2s, Hash as Blake2sHash, Params as Blake2sParams};

fn debug(_msg: &str) {}

use core::convert::TryInto;
use core::mem;
#[cfg(not(test))]
use core::panic::PanicInfo;

/*#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}*/

use crypto_api_chachapoly::{ChaCha20Ietf, ChachaPolyIetf}; //TODO: replace me with no-std version

const COMPACT_NOTE_SIZE: usize = (
    1  + // version
        11 + // diversifier
        8  + // value
        32
    // rcv
);
const NOTE_PLAINTEXT_SIZE: usize = COMPACT_NOTE_SIZE + 512;
const OUT_PLAINTEXT_SIZE: usize = (
    32 + // pk_d
        32
    // esk
);
const ENC_CIPHERTEXT_SIZE: usize = NOTE_PLAINTEXT_SIZE + 16;
const OUT_CIPHERTEXT_SIZE: usize = OUT_PLAINTEXT_SIZE + 16;

pub fn generate_esk(buffer: [u8; 64]) -> [u8; 32] {
    //Rng.fill_bytes(&mut buffer); fill with random bytes
    let esk = Fr::from_bytes_wide(&buffer);
    esk.to_bytes()
}

pub fn derive_public(esk: [u8; 32], g_d: [u8; 32]) -> [u8; 32] {
    let p = AffinePoint::from_bytes(g_d).unwrap();
    let q = p.to_niels().multiply_bits(&esk);
    let t = AffinePoint::from(q);
    t.to_bytes()
}

pub fn sapling_ka_agree(esk: [u8; 32], pk_d: [u8; 32]) -> [u8; 32] {
    let p = AffinePoint::from_bytes(pk_d).unwrap();
    let q = p.mul_by_cofactor();
    let v = q.to_niels().multiply_bits(&esk);
    let t = AffinePoint::from(v);
    t.to_bytes()
}

fn kdf_sapling(dhsecret: [u8; 32], epk: [u8; 32]) -> [u8; 32] {
    let mut input = [0u8; 64];
    (&mut input[..32]).copy_from_slice(&dhsecret);
    (&mut input[32..]).copy_from_slice(&epk);
    bolos::blake2b_kdf_sapling(&input)
}

fn prf_ock(
    ovk: [u8;32],
    cv: [u8;32],
    cmu: [u8;32],
    epk: [u8;32], ) -> [u8;32] {
    let mut ock_input = [0u8; 128];
    ock_input[0..32].copy_from_slice(&ovk); //Todo: compute this from secret key
    ock_input[32..64].copy_from_slice(&cv);
    ock_input[64..96].copy_from_slice(&cmu);
    ock_input[96..128].copy_from_slice(&epk);

    bolos::blake2b_prf_ock(&ock_input)
}
/*

def group_hash(D, M):
    digest = blake2s(person=D)
    digest.update(URS)
    digest.update(M)
    p = Point.from_bytes(digest.digest())
    if p is None:
        return None
    q = p * JUBJUB_COFACTOR
    if q == Point.ZERO:
        return None
    return q

def find_group_hash(D, M):
    i = 0
    while True:
        p = group_hash(D, M + bytes([i]))
        if p is not None:
            return p
        i += 1
        assert i < 256

def I_D_i(D, i):
    return find_group_hash(D, i2leosp(32, i - 1))

fn encode_chunk():
(s0, s1, s2) = mj
return (1 - 2*s2) * (1 + s0 + 2*s1)

def encode_segment(Mi):
ki = len(Mi) // 3
Michunks = [Mi[i:i+3] for i in range(0, len(Mi), 3)]
assert len(Michunks) == ki
return Fr(sum([encode_chunk(Michunks[j-1]) * 2**(4*(j-1)) for j in range(1, ki + 1)]))

c = 63



fn pedersen_hash_to_point(){
    Mdash = M + [0] * ((-len(M)) % 3)
    assert (len(Mdash) // 3) * 3 == len(Mdash)
    n = cldiv(len(Mdash), 3 * c)
    Msegs = [Mdash[i:i+(3*c)] for i in range(0, len(Mdash), 3*c)]
    assert len(Msegs) == n
    return sum([I_D_i(D, i) * encode_segment(Msegs[i-1]) for i in range(1, n + 1)], Point.ZERO)
}

def pedersen_hash_to_point(D, M):
# Pad M to a multiple of 3 bits
Mdash = M + [0] * ((-len(M)) % 3)
assert (len(Mdash) // 3) * 3 == len(Mdash)
n = cldiv(len(Mdash), 3 * c)
Msegs = [Mdash[i:i+(3*c)] for i in range(0, len(Mdash), 3*c)]
assert len(Msegs) == n
return sum([I_D_i(D, i) * encode_segment(Msegs[i-1]) for i in range(1, n + 1)], Point.ZERO)
*/

fn chacha_encryptnote(
    key: [u8; 32],
    plaintext: [u8; NOTE_PLAINTEXT_SIZE],
) -> [u8; ENC_CIPHERTEXT_SIZE] {
    let mut output = [0u8; ENC_CIPHERTEXT_SIZE];
    ChachaPolyIetf::aead_cipher()
        .seal_to(&mut output, &plaintext, &[], &key, &[0u8; 12])
        .unwrap();
    output
}

fn chacha_decryptnote(
    key: [u8; 32],
    ciphertext: [u8; ENC_CIPHERTEXT_SIZE],
) -> [u8; NOTE_PLAINTEXT_SIZE] {
    let mut plaintext = [0u8; NOTE_PLAINTEXT_SIZE];
    ChachaPolyIetf::aead_cipher()
        .open_to(&mut plaintext, &ciphertext, &[], &key, &[0u8; 12])
        .unwrap();
    plaintext
}

#[inline(always)]
pub fn prf_expand(sk: &[u8], t: &[u8]) -> [u8; 64] {
    bolos::blake2b_expand_seed(sk, t)
}

fn sapling_derive_dummy_ask(sk_in: &[u8]) -> [u8; 32] {
    let t = prf_expand(&sk_in, &[0x00]);
    let ask = Fr::from_bytes_wide(&t);
    ask.to_bytes()
}

fn sapling_derive_dummy_nsk(sk_in: &[u8]) -> [u8; 32] {
    let t = prf_expand(&sk_in, &[0x01]);
    let nsk = Fr::from_bytes_wide(&t);
    nsk.to_bytes()
}

fn sapling_ask_to_ak(ask: &[u8; 32]) -> [u8; 32] {
    let ak = constants::SPENDING_KEY_BASE.multiply_bits(&ask);
    AffinePoint::from(ak).to_bytes()
}

fn sapling_nsk_to_nk(nsk: &[u8; 32]) -> [u8; 32] {
    let nk = constants::PROVING_KEY_BASE.multiply_bits(&nsk);
    AffinePoint::from(nk).to_bytes()
}

fn aknk_to_ivk(ak: &[u8; 32], nk: &[u8; 32]) -> [u8; 32] {
    pub const CRH_IVK_PERSONALIZATION: &[u8; 8] = b"Zcashivk"; //move to constants

    // blake2s CRH_IVK_PERSONALIZATION || ak || nk
    let h = Blake2sParams::new()
        .hash_length(32)
        .personal(CRH_IVK_PERSONALIZATION)
        .to_state()
        .update(ak)
        .update(nk)
        .finalize();

    let mut x: [u8; 32] = *h.as_array();
    x[31] &= 0b0000_0111; //check this
    x
}

#[inline(never)]
fn group_hash_check(hash: &[u8; 32]) -> bool {
    let u = AffinePoint::from_bytes(*hash);
    if u.is_some().unwrap_u8() == 1 {
        let v = u.unwrap();
        let q = v.mul_by_cofactor();
        let i = ExtendedPoint::identity();
        return q != i;
    }

    false
}

#[inline(never)]
fn diversifier_group_hash_light(tag: &[u8]) -> bool {
    let x = bolos::blake2s_diversification(tag);

    //    diversifier_group_hash_check(&x)

    let u = AffinePoint::from_bytes(x);
    if u.is_some().unwrap_u8() == 1 {
        let v = u.unwrap();
        let q = v.mul_by_cofactor();
        let i = ExtendedPoint::identity();
        return q != i;
    }

    false
}

#[inline(never)]
fn default_diversifier(sk: &[u8; 32]) -> [u8; 11] {
    //fixme: replace blake2b with aes
    let mut c: [u8; 2] = [0x03, 0x0];

    // blake2b sk || 0x03 || c
    loop {
        let x = prf_expand(sk, &c);
        if diversifier_group_hash_light(&x[0..11]) {
            let mut result = [0u8; 11];
            result.copy_from_slice(&x[..11]);
            return result;
        }
        c[1] += 1;
    }
}

#[inline(never)]
fn pkd_group_hash(d: &[u8; 11]) -> [u8; 32] {
    let h = bolos::blake2s_diversification(d);

    let v = AffinePoint::from_bytes(h).unwrap();
    let q = v.mul_by_cofactor();
    let t = AffinePoint::from(q);
    t.to_bytes()
}

#[inline(never)]
fn default_pkd(ivk: &[u8; 32], d: &[u8; 11]) -> [u8; 32] {
    let h = bolos::blake2s_diversification(d);

    let v = AffinePoint::from_bytes(h).unwrap();
    let y = v.mul_by_cofactor();

    // FIXME: We should avoid asserts in ledger code
    //assert_eq!(x.is_some().unwrap_u8(), 1);

    let v = y.to_niels().multiply_bits(ivk);
    let t = AffinePoint::from(v);
    t.to_bytes()
}

#[no_mangle]
pub extern "C" fn get_ak(sk_ptr: *mut u8, ak_ptr: *mut u8) {
    let sk: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(sk_ptr) };
    let ak: &mut [u8; 32] = unsafe { mem::transmute::<*const u8, &mut [u8; 32]>(ak_ptr) };
    let ask = sapling_derive_dummy_ask(sk);
    let tmp_ak = sapling_ask_to_ak(&ask);
    ak.copy_from_slice(&tmp_ak)
}

#[no_mangle]
pub extern "C" fn get_nk(sk_ptr: *mut u8, nk_ptr: *mut u8) {
    let sk: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(sk_ptr) };
    let nk: &mut [u8; 32] = unsafe { mem::transmute::<*const u8, &mut [u8; 32]>(nk_ptr) };
    let nsk = sapling_derive_dummy_nsk(sk);
    let tmp_nk = sapling_nsk_to_nk(&nsk);
    nk.copy_from_slice(&tmp_nk)
}

#[no_mangle]
pub extern "C" fn get_ivk(ak_ptr: *mut u8, nk_ptr: *mut u8, ivk_ptr: *mut u8) {
    let ak: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(ak_ptr) };
    let nk: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(nk_ptr) };
    let ivk: &mut [u8; 32] = unsafe { mem::transmute::<*const u8, &mut [u8; 32]>(ivk_ptr) };

    let tmp_ivk = aknk_to_ivk(&ak, &nk);
    ivk.copy_from_slice(&tmp_ivk)
}

#[no_mangle]
pub extern "C" fn get_diversifier(sk_ptr: *mut u8, diversifier_ptr: *mut u8) {
    let sk: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(sk_ptr) };
    let diversifier: &mut [u8; 11] =
        unsafe { mem::transmute::<*const u8, &mut [u8; 11]>(diversifier_ptr) };
    let d = default_diversifier(sk);
    diversifier.copy_from_slice(&d)
}

#[no_mangle]
pub extern "C" fn get_pkd(ivk_ptr: *mut u8, diversifier_ptr: *mut u8, pkd_ptr: *mut u8) {
    let ivk: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(ivk_ptr) };
    let diversifier: &[u8; 11] = unsafe { mem::transmute::<*const u8, &[u8; 11]>(diversifier_ptr) };
    let pkd: &mut [u8; 32] = unsafe { mem::transmute::<*const u8, &mut [u8; 32]>(pkd_ptr) };

    let tmp_pkd = default_pkd(&ivk, &diversifier);
    pkd.copy_from_slice(&tmp_pkd)
}

//fixme
//fixme: we need to add a prefix to exported functions.. as there are no namespaces in C :(
//get seed from the ledger
#[no_mangle]
pub extern "C" fn get_address(sk_ptr: *mut u8, ivk_ptr: *mut u8, address_ptr: *mut u8) {
    let sk: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(sk_ptr) };
    let ivk: &[u8; 32] = unsafe { mem::transmute::<*const u8, &[u8; 32]>(ivk_ptr) };
    let address: &mut [u8; 43] = unsafe { mem::transmute::<*const u8, &mut [u8; 43]>(address_ptr) };

    let div = default_diversifier(sk);
    let pkd = default_pkd(&ivk, &div);

    address[..11].copy_from_slice(&div);
    address[11..].copy_from_slice(&pkd);
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_sharedsecret() {
        let esk: [u8; 32] = [
            0x81, 0xc7, 0xb2, 0x17, 0x1f, 0xf4, 0x41, 0x52, 0x50, 0xca, 0xc0, 0x1f, 0x59, 0x82,
            0xfd, 0x8f, 0x49, 0x61, 0x9d, 0x61, 0xad, 0x78, 0xf6, 0x83, 0x0b, 0x3c, 0x60, 0x61,
            0x45, 0x96, 0x2a, 0x0e,
        ];
        let pk_d: [u8; 32] = [
            0x88, 0x99, 0xc6, 0x44, 0xbf, 0xc6, 0x0f, 0x87, 0x83, 0xf9, 0x2b, 0xa9, 0xf8, 0x18,
            0x9e, 0xd2, 0x77, 0xbf, 0x68, 0x3d, 0x5d, 0x1d, 0xae, 0x02, 0xc5, 0x71, 0xff, 0x47,
            0x86, 0x9a, 0x0b, 0xa6,
        ];
        let sharedsecret: [u8; 32] = [
            0x2e, 0x35, 0x7d, 0x82, 0x2e, 0x02, 0xdc, 0xe8, 0x84, 0xee, 0x94, 0x8a, 0xb4, 0xff,
            0xb3, 0x20, 0x6b, 0xa5, 0x74, 0x77, 0xac, 0x7d, 0x7b, 0x07, 0xed, 0x44, 0x6c, 0x3b,
            0xe4, 0x48, 0x1b, 0x3e,
        ];
        assert_eq!(sapling_ka_agree(esk, pk_d), sharedsecret);
    }

    #[test]
    fn test_encryption() {
        let k_enc = [
            0x6d, 0xf8, 0x5b, 0x17, 0x89, 0xb0, 0xb7, 0x8b, 0x46, 0x10, 0xf2, 0x5d, 0x36, 0x8c,
            0xb5, 0x11, 0x14, 0x0a, 0x7c, 0x0a, 0xf3, 0xbc, 0x3d, 0x2a, 0x22, 0x6f, 0x92, 0x7d,
            0xe6, 0x02, 0xa7, 0xf1,
        ];
        let p_enc = [
            0x01, 0xdc, 0xe7, 0x7e, 0xbc, 0xec, 0x0a, 0x26, 0xaf, 0xd6, 0x99, 0x8c, 0x00, 0xe1,
            0xf5, 0x05, 0x00, 0x00, 0x00, 0x00, 0x39, 0x17, 0x6d, 0xac, 0x39, 0xac, 0xe4, 0x98,
            0x0e, 0xcc, 0x8d, 0x77, 0x8e, 0x89, 0x86, 0x02, 0x55, 0xec, 0x36, 0x15, 0x06, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf6, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            0x00, 0x00, 0x00, 0x00,
        ];

        let c_enc = [
            0xbd, 0xcb, 0x94, 0x72, 0xa1, 0xac, 0xad, 0xf1, 0xd0, 0x82, 0x07, 0xf6, 0x3c, 0xaf,
            0x4f, 0x3a, 0x76, 0x3c, 0x67, 0xd0, 0x66, 0x56, 0x0a, 0xd9, 0x6c, 0x1e, 0xf9, 0x52,
            0xf8, 0x46, 0xa9, 0xc2, 0x80, 0x82, 0xdd, 0xef, 0x45, 0x21, 0xf6, 0x82, 0x54, 0x76,
            0xad, 0xe3, 0x2e, 0xeb, 0x34, 0x64, 0x06, 0xa5, 0xee, 0xc9, 0x4b, 0x4a, 0xb9, 0xe4,
            0x55, 0x12, 0x42, 0xb1, 0x44, 0xa4, 0xf8, 0xc8, 0x28, 0xbc, 0x19, 0x7f, 0x3e, 0x92,
            0x5f, 0x61, 0x7f, 0xc4, 0xb9, 0xc1, 0xb1, 0x53, 0xad, 0x15, 0x3a, 0x3c, 0x56, 0xf8,
            0x1f, 0xc4, 0x8b, 0xf5, 0x4e, 0x6e, 0xe8, 0x89, 0x5f, 0x27, 0x8c, 0x5e, 0x4c, 0x6a,
            0xe7, 0xa8, 0xa0, 0x23, 0x86, 0x70, 0x85, 0xb4, 0x07, 0xbe, 0xce, 0x40, 0x0b, 0xc6,
            0xaa, 0xec, 0x06, 0xaf, 0xf8, 0xb0, 0x49, 0xbc, 0xb2, 0x63, 0x63, 0xc6, 0xde, 0x01,
            0x8d, 0x2d, 0xa0, 0x41, 0xcc, 0x2e, 0xb8, 0xd0, 0x86, 0x4a, 0x70, 0xdf, 0x68, 0x47,
            0xb3, 0x37, 0x5a, 0x31, 0x86, 0x6c, 0x49, 0xa8, 0x02, 0x5a, 0xd7, 0x17, 0xe7, 0x79,
            0xbd, 0x0f, 0xb5, 0xce, 0xed, 0x3e, 0xc4, 0x40, 0x8e, 0x18, 0x50, 0x69, 0x4b, 0xa3,
            0x56, 0x39, 0xdd, 0x8b, 0x55, 0xd2, 0xbf, 0xdf, 0xc6, 0x40, 0x6c, 0x78, 0xc0, 0x0e,
            0xb5, 0xfc, 0x48, 0x76, 0x4b, 0xf4, 0xd8, 0x4d, 0xe1, 0xa0, 0x26, 0xd9, 0x02, 0x86,
            0x60, 0xa9, 0xa5, 0xc1, 0xc5, 0x94, 0xb8, 0x15, 0x8c, 0x69, 0x1e, 0x50, 0x68, 0xc8,
            0x51, 0xda, 0xfa, 0x30, 0x10, 0xe3, 0x9b, 0x70, 0xc4, 0x66, 0x83, 0x73, 0xbb, 0x59,
            0xac, 0x53, 0x07, 0x0c, 0x7b, 0x3f, 0x76, 0x62, 0x03, 0x84, 0x27, 0xb3, 0x72, 0xfd,
            0x75, 0x36, 0xe5, 0x4d, 0x8c, 0x8e, 0x61, 0x56, 0x2c, 0xb0, 0xe5, 0x7e, 0xf7, 0xb4,
            0x43, 0xde, 0x5e, 0x47, 0x8f, 0x4b, 0x02, 0x9c, 0x36, 0xaf, 0x71, 0x27, 0x1a, 0x0f,
            0x9d, 0x57, 0xbe, 0x80, 0x1b, 0xc4, 0xf2, 0x61, 0x8d, 0xc4, 0xf0, 0xab, 0xd1, 0x5f,
            0x0b, 0x42, 0x0c, 0x11, 0x14, 0xbb, 0xd7, 0x27, 0xe4, 0xb3, 0x1a, 0x6a, 0xaa, 0xd8,
            0xfe, 0x53, 0xb7, 0xdf, 0x60, 0xb4, 0xe0, 0xc9, 0xe9, 0x45, 0x7b, 0x89, 0x3f, 0x20,
            0xec, 0x18, 0x61, 0x1e, 0x68, 0x03, 0x05, 0xfe, 0x04, 0xba, 0x3b, 0x8d, 0x30, 0x1f,
            0x5c, 0xd8, 0x2c, 0x2c, 0x8d, 0x1c, 0x58, 0x5d, 0x51, 0x15, 0x4b, 0x46, 0x88, 0xff,
            0x5a, 0x35, 0x0b, 0x60, 0xae, 0x30, 0xda, 0x4f, 0x74, 0xc3, 0xd5, 0x5c, 0x73, 0xda,
            0xe8, 0xad, 0x9a, 0xb8, 0x0b, 0xbb, 0x5d, 0xdf, 0x1b, 0xea, 0xec, 0x12, 0x0f, 0xc4,
            0xf7, 0x8d, 0xe5, 0x4f, 0xef, 0xe1, 0xa8, 0x41, 0x35, 0x79, 0xfd, 0xce, 0xa2, 0xf6,
            0x56, 0x74, 0x10, 0x4c, 0xba, 0xac, 0x7e, 0x0d, 0xe5, 0x08, 0x3d, 0xa7, 0xb1, 0xb7,
            0xf2, 0xe9, 0x43, 0x70, 0xdd, 0x0a, 0x3e, 0xed, 0x71, 0x50, 0x36, 0x54, 0x2f, 0xa4,
            0x0e, 0xd4, 0x89, 0x2b, 0xaa, 0xfb, 0x57, 0x2e, 0xe0, 0xf9, 0x45, 0x9c, 0x1c, 0xbe,
            0x3a, 0xd1, 0xb6, 0xaa, 0xf1, 0x1f, 0x54, 0x93, 0x59, 0x52, 0xbe, 0x6b, 0x95, 0x38,
            0xa9, 0xa3, 0x9e, 0xde, 0x64, 0x2b, 0xb0, 0xcd, 0xac, 0x1c, 0x09, 0x09, 0x2c, 0xd7,
            0x11, 0x16, 0x0a, 0x8d, 0x45, 0x19, 0xb4, 0xce, 0x20, 0xff, 0xf6, 0x61, 0x2b, 0xc7,
            0xb0, 0x53, 0x93, 0xbb, 0x7e, 0x96, 0xf8, 0xea, 0x4b, 0xbc, 0x97, 0x83, 0x1f, 0x20,
            0x46, 0xe1, 0xcb, 0x5a, 0x2c, 0xe7, 0xca, 0x36, 0xfd, 0x06, 0xab, 0x39, 0x56, 0xa8,
            0x03, 0xd4, 0x32, 0x5a, 0xae, 0x72, 0xef, 0xb7, 0x07, 0xca, 0xa0, 0x44, 0xd3, 0xf8,
            0xfc, 0x7d, 0x09, 0x46, 0xbe, 0xb1, 0x1c, 0xdd, 0xc8, 0x53, 0xdb, 0xcf, 0x24, 0x3a,
            0xf3, 0xe5, 0x92, 0xb8, 0x1d, 0xb3, 0x64, 0x19, 0xd3, 0x4a, 0x4b, 0xb1, 0xee, 0x53,
            0xc1, 0xa1, 0xba, 0x51, 0xc1, 0x8b, 0x2e, 0xe9, 0x2d, 0xb4, 0xbf, 0x5f, 0xce, 0xeb,
            0x82, 0x0e, 0x8c, 0x58, 0xf8, 0x16, 0x6c, 0x3a, 0xcb, 0xf7, 0x61, 0xb5, 0xb1, 0xf2,
            0x9c, 0x3f, 0x11, 0x81, 0x67, 0xbb, 0x6c, 0xdb, 0x23, 0x30, 0x35, 0x29, 0x6a, 0xd4,
            0x0e, 0x8a, 0xa0, 0xce, 0xf5, 0x70,
        ];
        assert_eq!(chacha_encryptnote(k_enc, p_enc)[0..32], c_enc[0..32]);
        assert_eq!(chacha_decryptnote(k_enc, c_enc)[0..32], p_enc[0..32]);
    }

    #[test]
    fn test_kdf() {
        let esk: [u8; 32] = [
            0x81, 0xc7, 0xb2, 0x17, 0x1f, 0xf4, 0x41, 0x52, 0x50, 0xca, 0xc0, 0x1f, 0x59, 0x82,
            0xfd, 0x8f, 0x49, 0x61, 0x9d, 0x61, 0xad, 0x78, 0xf6, 0x83, 0x0b, 0x3c, 0x60, 0x61,
            0x45, 0x96, 0x2a, 0x0e,
        ];
        let g_d = pkd_group_hash(&[
            0xdc, 0xe7, 0x7e, 0xbc, 0xec, 0x0a, 0x26, 0xaf, 0xd6, 0x99, 0x8c,
        ]);
        let dp = derive_public(esk, g_d);

        let epk: [u8; 32] = [
            0x7e, 0xb9, 0x28, 0xf9, 0xf6, 0xd5, 0x96, 0xbf, 0xbf, 0x81, 0x4e, 0x3d, 0xd0, 0xe2,
            0x4f, 0xdc, 0x52, 0x03, 0x0f, 0xd1, 0x0f, 0x49, 0x0b, 0xa2, 0x04, 0x58, 0x68, 0xda,
            0x98, 0xf3, 0x49, 0x36,
        ];
        assert_eq!(dp, epk);
        let k_enc = [
            0x6d, 0xf8, 0x5b, 0x17, 0x89, 0xb0, 0xb7, 0x8b, 0x46, 0x10, 0xf2, 0x5d, 0x36, 0x8c,
            0xb5, 0x11, 0x14, 0x0a, 0x7c, 0x0a, 0xf3, 0xbc, 0x3d, 0x2a, 0x22, 0x6f, 0x92, 0x7d,
            0xe6, 0x02, 0xa7, 0xf1,
        ];
        let sharedsecret: [u8; 32] = [
            0x2e, 0x35, 0x7d, 0x82, 0x2e, 0x02, 0xdc, 0xe8, 0x84, 0xee, 0x94, 0x8a, 0xb4, 0xff,
            0xb3, 0x20, 0x6b, 0xa5, 0x74, 0x77, 0xac, 0x7d, 0x7b, 0x07, 0xed, 0x44, 0x6c, 0x3b,
            0xe4, 0x48, 0x1b, 0x3e,
        ];
        assert_eq!(kdf_sapling(sharedsecret, epk), k_enc);
    }

    #[test]
    fn test_ock(){
        //prf_ock(ovk, cv, cmu, ephemeral_key)
        let ovk :[u8;32] = [
            0x98, 0xd1, 0x69, 0x13, 0xd9, 0x9b, 0x04, 0x17, 0x7c, 0xab, 0xa4, 0x4f, 0x6e, 0x4d, 0x22, 0x4e, 0x03, 0xb5, 0xac, 0x03, 0x1d, 0x7c, 0xe4, 0x5e, 0x86, 0x51, 0x38, 0xe1, 0xb9, 0x96, 0xd6, 0x3b
        ];

        let cv: [u8;32] = [
            0xa9, 0xcb, 0x0d, 0x13, 0x72, 0x32, 0xff, 0x84, 0x48, 0xd0, 0xf0, 0x78, 0xb6, 0x81, 0x4c, 0x66, 0xcb, 0x33, 0x1b, 0x0f, 0x2d, 0x3d, 0x8a, 0x08, 0x5b, 0xed, 0xba, 0x81, 0x5f, 0x00, 0xa8, 0xdb
        ];

        let cmu: [u8;32] = [
            0x8d, 0xe2, 0xc9, 0xb3, 0xf9, 0x14, 0x67, 0xd5, 0x14, 0xfe, 0x2f, 0x97, 0x42, 0x2c, 0x4f, 0x76, 0x11, 0xa9, 0x1b, 0xb7, 0x06, 0xed, 0x5c, 0x27, 0x72, 0xd9, 0x91, 0x22, 0xa4, 0x21, 0xe1, 0x2d
        ];

        let epk: [u8;32] = [
            0x7e, 0xb9, 0x28, 0xf9, 0xf6, 0xd5, 0x96, 0xbf, 0xbf, 0x81, 0x4e, 0x3d, 0xd0, 0xe2, 0x4f, 0xdc, 0x52, 0x03, 0x0f, 0xd1, 0x0f, 0x49, 0x0b, 0xa2, 0x04, 0x58, 0x68, 0xda, 0x98, 0xf3, 0x49, 0x36
        ];

        let ock: [u8;32] = [
            0x41, 0x14, 0x43, 0xfc, 0x1d, 0x92, 0x54, 0x33, 0x74, 0x15, 0xb2, 0x14, 0x7a, 0xde, 0xcd, 0x48, 0xf3, 0x13, 0x76, 0x9c, 0x3b, 0xa1, 0x77, 0xd4, 0xcd, 0x34, 0xd6, 0xfb, 0xd1, 0x40, 0x27, 0x0d
        ];

        assert_eq!(prf_ock(ovk,cv,cmu,epk),ock);
    }

    #[test]
    fn test_div() {
        let nk = [
            0xf7, 0xcf, 0x9e, 0x77, 0xf2, 0xe5, 0x86, 0x83, 0x38, 0x3c, 0x15, 0x19, 0xac, 0x7b,
            0x06, 0x2d, 0x30, 0x04, 0x0e, 0x27, 0xa7, 0x25, 0xfb, 0x88, 0xfb, 0x19, 0xa9, 0x78,
            0xbd, 0x3f, 0xd6, 0xba,
        ];
        let ak = [
            0xf3, 0x44, 0xec, 0x38, 0x0f, 0xe1, 0x27, 0x3e, 0x30, 0x98, 0xc2, 0x58, 0x8c, 0x5d,
            0x3a, 0x79, 0x1f, 0xd7, 0xba, 0x95, 0x80, 0x32, 0x76, 0x07, 0x77, 0xfd, 0x0e, 0xfa,
            0x8e, 0xf1, 0x16, 0x20,
        ];

        let ivk: [u8; 32] = aknk_to_ivk(&ak, &nk);
        let default_d = [
            0xf1, 0x9d, 0x9b, 0x79, 0x7e, 0x39, 0xf3, 0x37, 0x44, 0x58, 0x39,
        ];

        let result = pkd_group_hash(&default_d);
        let x = super::AffinePoint::from_bytes(result);
        if x.is_some().unwrap_u8() == 1 {
            let y = super::ExtendedPoint::from(x.unwrap());
            let v = y.to_niels().multiply_bits(&ivk);
            let t = super::AffinePoint::from(v);
            let pk_d = t.to_bytes();
            assert_eq!(
                pk_d,
                [
                    0xdb, 0x4c, 0xd2, 0xb0, 0xaa, 0xc4, 0xf7, 0xeb, 0x8c, 0xa1, 0x31, 0xf1, 0x65,
                    0x67, 0xc4, 0x45, 0xa9, 0x55, 0x51, 0x26, 0xd3, 0xc2, 0x9f, 0x14, 0xe3, 0xd7,
                    0x76, 0xe8, 0x41, 0xae, 0x74, 0x15
                ]
            );
        }
    }

    #[test]
    fn test_default_diversifier() {
        let seed = [0u8; 32];
        let default_d = default_diversifier(&seed);
        assert_eq!(
            default_d,
            [0xf1, 0x9d, 0x9b, 0x79, 0x7e, 0x39, 0xf3, 0x37, 0x44, 0x58, 0x39]
        );
    }

    #[test]
    fn test_defaultpkd() {
        let seed = [0u8; 32];
        let default_d = default_diversifier(&seed);

        let nk = [
            0xf7, 0xcf, 0x9e, 0x77, 0xf2, 0xe5, 0x86, 0x83, 0x38, 0x3c, 0x15, 0x19, 0xac, 0x7b,
            0x06, 0x2d, 0x30, 0x04, 0x0e, 0x27, 0xa7, 0x25, 0xfb, 0x88, 0xfb, 0x19, 0xa9, 0x78,
            0xbd, 0x3f, 0xd6, 0xba,
        ];
        let ak = [
            0xf3, 0x44, 0xec, 0x38, 0x0f, 0xe1, 0x27, 0x3e, 0x30, 0x98, 0xc2, 0x58, 0x8c, 0x5d,
            0x3a, 0x79, 0x1f, 0xd7, 0xba, 0x95, 0x80, 0x32, 0x76, 0x07, 0x77, 0xfd, 0x0e, 0xfa,
            0x8e, 0xf1, 0x16, 0x20,
        ];

        let ivk: [u8; 32] = aknk_to_ivk(&ak, &nk);

        let pkd = default_pkd(&ivk, &default_d);
        assert_eq!(
            pkd,
            [
                0xdb, 0x4c, 0xd2, 0xb0, 0xaa, 0xc4, 0xf7, 0xeb, 0x8c, 0xa1, 0x31, 0xf1, 0x65, 0x67,
                0xc4, 0x45, 0xa9, 0x55, 0x51, 0x26, 0xd3, 0xc2, 0x9f, 0x14, 0xe3, 0xd7, 0x76, 0xe8,
                0x41, 0xae, 0x74, 0x15
            ]
        );
    }

    #[test]
    fn test_grouphash_default() {
        let default_d = [
            0xf1, 0x9d, 0x9b, 0x79, 0x7e, 0x39, 0xf3, 0x37, 0x44, 0x58, 0x39,
        ];

        let result = pkd_group_hash(&default_d);
        let x = super::AffinePoint::from_bytes(result);
        assert_eq!(x.is_some().unwrap_u8(), 1);
        assert_eq!(
            result,
            [
                0x3a, 0x71, 0xe3, 0x48, 0x16, 0x9e, 0x0c, 0xed, 0xbc, 0x4f, 0x36, 0x33, 0xa2, 0x60,
                0xd0, 0xe7, 0x85, 0xea, 0x8f, 0x89, 0x27, 0xce, 0x45, 0x01, 0xce, 0xf3, 0x21, 0x6e,
                0xd0, 0x75, 0xce, 0xa2
            ]
        );
    }

    #[test]
    fn test_ak() {
        let seed = [0u8; 32];
        let ask: [u8; 32] = sapling_derive_dummy_ask(&seed);
        assert_eq!(
            ask,
            [
                0x85, 0x48, 0xa1, 0x4a, 0x47, 0x3e, 0xa5, 0x47, 0xaa, 0x23, 0x78, 0x40, 0x20, 0x44,
                0xf8, 0x18, 0xcf, 0x19, 0x11, 0xcf, 0x5d, 0xd2, 0x05, 0x4f, 0x67, 0x83, 0x45, 0xf0,
                0x0d, 0x0e, 0x88, 0x06
            ]
        );
        let ak: [u8; 32] = sapling_ask_to_ak(&ask);
        assert_eq!(
            ak,
            [
                0xf3, 0x44, 0xec, 0x38, 0x0f, 0xe1, 0x27, 0x3e, 0x30, 0x98, 0xc2, 0x58, 0x8c, 0x5d,
                0x3a, 0x79, 0x1f, 0xd7, 0xba, 0x95, 0x80, 0x32, 0x76, 0x07, 0x77, 0xfd, 0x0e, 0xfa,
                0x8e, 0xf1, 0x16, 0x20
            ]
        );
    }

    #[test]
    fn test_nk() {
        let seed = [0u8; 32];

        let nsk: [u8; 32] = sapling_derive_dummy_nsk(&seed);
        assert_eq!(
            nsk,
            [
                0x30, 0x11, 0x4e, 0xa0, 0xdd, 0x0b, 0xb6, 0x1c, 0xf0, 0xea, 0xea, 0xb6, 0xec, 0x33,
                0x31, 0xf5, 0x81, 0xb0, 0x42, 0x5e, 0x27, 0x33, 0x85, 0x01, 0x26, 0x2d, 0x7e, 0xac,
                0x74, 0x5e, 0x6e, 0x05
            ]
        );

        let nk: [u8; 32] = sapling_nsk_to_nk(&nsk);
        assert_eq!(
            nk,
            [
                0xf7, 0xcf, 0x9e, 0x77, 0xf2, 0xe5, 0x86, 0x83, 0x38, 0x3c, 0x15, 0x19, 0xac, 0x7b,
                0x06, 0x2d, 0x30, 0x04, 0x0e, 0x27, 0xa7, 0x25, 0xfb, 0x88, 0xfb, 0x19, 0xa9, 0x78,
                0xbd, 0x3f, 0xd6, 0xba
            ]
        );
    }

    #[test]
    fn test_ivk() {
        let nk = [
            0xf7, 0xcf, 0x9e, 0x77, 0xf2, 0xe5, 0x86, 0x83, 0x38, 0x3c, 0x15, 0x19, 0xac, 0x7b,
            0x06, 0x2d, 0x30, 0x04, 0x0e, 0x27, 0xa7, 0x25, 0xfb, 0x88, 0xfb, 0x19, 0xa9, 0x78,
            0xbd, 0x3f, 0xd6, 0xba,
        ];
        let ak = [
            0xf3, 0x44, 0xec, 0x38, 0x0f, 0xe1, 0x27, 0x3e, 0x30, 0x98, 0xc2, 0x58, 0x8c, 0x5d,
            0x3a, 0x79, 0x1f, 0xd7, 0xba, 0x95, 0x80, 0x32, 0x76, 0x07, 0x77, 0xfd, 0x0e, 0xfa,
            0x8e, 0xf1, 0x16, 0x20,
        ];

        let ivk: [u8; 32] = aknk_to_ivk(&ak, &nk);
        assert_eq!(
            ivk,
            [
                0xb7, 0x0b, 0x7c, 0xd0, 0xed, 0x03, 0xcb, 0xdf, 0xd7, 0xad, 0xa9, 0x50, 0x2e, 0xe2,
                0x45, 0xb1, 0x3e, 0x56, 0x9d, 0x54, 0xa5, 0x71, 0x9d, 0x2d, 0xaa, 0x0f, 0x5f, 0x14,
                0x51, 0x47, 0x92, 0x04
            ]
        );
    }
}
