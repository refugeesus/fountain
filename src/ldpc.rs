use labrador_ldpc::LDPCCode;
use crate::droplet::Droplet;

#[derive(Debug)]
pub enum DecoderType {
    Ms,
    Bf,
}

pub fn droplet_decode(droplet: &mut Droplet, code: LDPCCode, decoder: DecoderType) {
    match decoder {
        DecoderType::Bf => {
            // Allocate some memory for the decoder's working area and output
            let mut working = vec![0u8; code.decode_bf_working_len()];
            let mut rxdata = vec![0u8; code.output_len()];
            code.decode_bf(&droplet.data, &mut rxdata, &mut working, 50);

            droplet.data.resize(rxdata.len(), 0);
            droplet.data.clear();
            droplet.data = rxdata.clone();
        }
        DecoderType::Ms => {
            // Allocate ms working memory and output
            let mut working = vec![0i8; code.decode_ms_working_len()];
            let mut working_u8 = vec![0u8; code.output_len() - code.k()/8];
            let mut llrs = vec![0i8; code.n()];
            let mut output = vec![0u8; code.output_len()];
            // Create soft llrs from hard bits
            code.hard_to_llrs(&droplet.data, &mut llrs);
            code.decode_ms(&llrs, &mut output, &mut working, &mut working_u8, 50);

            droplet.data.clear();
            droplet.data = output.clone();
        }
    }
}

pub fn droplet_encode(droplet: &mut Droplet, code: LDPCCode) {
    let mut txcode = vec![0u8; code.n() / 8];
    code.copy_encode(&droplet.data, &mut txcode);
    droplet.data.resize(txcode.len(), 0);
    droplet.data.clear();
    droplet.data = txcode.clone();
}
