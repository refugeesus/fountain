use labrador_ldpc::LDPCCode;
use crate::droplet::Droplet;

pub fn droplet_decode(droplet: &mut Droplet, code: LDPCCode) {
    // Allocate some memory for the decoder's working area and output
    let mut working = vec![0u8; code.decode_bf_working_len()];
    let mut rxdata = vec![0u8; code.output_len()];
    code.decode_bf(&droplet.data, &mut rxdata, &mut working, 20);

    droplet.data.resize(rxdata.len(), 0);
    droplet.data.clear();
    droplet.data = rxdata.clone();

    // Integrate this form of decoding for a more robust recovery scenario. Unfortunately this does
    // require more overhead and working space but the recovery rate for block erasures is
    // significantly better
    /*
    let mut working = vec![0i8; code.decode_ms_working_len()];
    let mut working_u8 = vec![0u8; code.decode_ms_working_u8_len()];
    let mut rxdata = vec![0u8; code.output_len()];
    let mut llrs = vec![0i8; code.n()];
    let mut output = vec![0u8; code.n() / 8];
    code.hard_to_llrs(&pkt.payload, &mut llrs);
    code.decode_ms(&llrs, &mut rxdata, &mut working, &mut working_u8, 20);
    debug!("ms decoded: {:?}", &rxdata);
    code.llrs_to_hard(&llrs, &mut output);
    pkt.payload.clear();
    pkt.payload = output.clone();
    */
    //println!("decoded {:?}", &droplet.data);
}

pub fn droplet_encode(droplet: &mut Droplet, code: LDPCCode) {
    let mut txcode = vec![0u8; code.n() / 8];
    code.copy_encode(&droplet.data, &mut txcode);
    droplet.data.resize(txcode.len(), 0);
    droplet.data.clear();
    droplet.data = txcode.clone();
}
