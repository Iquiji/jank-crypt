use std::vec;

const SIZE: usize = 32;

pub fn usha0(message: &[u8]) -> Vec<u8> {
    let mut output = vec![6u64; SIZE];

    let mut data = message.to_vec();
    data.append(&mut message.len().to_be_bytes().to_vec());

    const CHUNK_SIZE: usize = 128;

    let to_be_padded =
        (((CHUNK_SIZE).wrapping_sub(data.len())) % CHUNK_SIZE + CHUNK_SIZE) % CHUNK_SIZE;

    data.append(&mut vec![0u8; to_be_padded]);

    // let k: [u8;6] = [0x34,0xAF,0x74,0x12,0xF3,0xC3];
    // data.append(&mut k.to_vec());

    let data_iter = data.chunks(CHUNK_SIZE);

    let mut extended_data: Vec<u64> = vec![];

    //println!("len of usha data mod CHUNK_SIZE: {}",data.len() % 32);

    for bytes_e in data_iter {
        let mut extender = vec![0u64; CHUNK_SIZE / 4 + CHUNK_SIZE / 8];
        for i in (0..CHUNK_SIZE).step_by(8) {
            extender[i / 8] = u64::from_be_bytes([
                bytes_e[i],
                bytes_e[i + 1],
                bytes_e[i + 2],
                bytes_e[i + 3],
                bytes_e[i + 4],
                bytes_e[i + 5],
                bytes_e[i + 6],
                bytes_e[i + 7],
            ]);
        }
        for i in (CHUNK_SIZE / 8)..(CHUNK_SIZE / 4) {
            extender[i] = extender[i - 2]
                ^ extender[i - 3].rotate_right(9)
                ^ extender[i - 7]
                ^ extender[i - 13].rotate_left(2);
            extender[i] = extender[i].rotate_left(5);
        }

        for i in 20..extender.len() {
            extender[i] ^= extender[i - 15].rotate_left(3) & 0xAF421F4D ^ extender[i - 19];
            extender[i - 3] = extender[i - 8].rotate_right(12) & 0xC35BD342;
            extender[i - 7] &= extender[i - 9].rotate_right(17) ^ 0x623144AF;
            extender[i - 2] = extender[i - 15].rotate_right(33) & 0xF3512FC1;
            extender[i - 6] = extender[i - 5].rotate_right(63) | 0xF3512FC1 ^ extender[i - 1];
            extender[i - 17] = extender[i - 19].rotate_left(42) ^ 0xF3512FC1 & extender[i - 15];
            extender[i - 3] = extender[i - 4].rotate_left(27);
            extender[i - 9] = extender[i] ^ extender[i - 5] ^ extender[i - 7];
        }

        for ext in extender {
            extended_data.push(ext);
        }
    }

    for _ in 0..32 {
        for (i, byte) in extended_data.iter().enumerate() {
            output[i % SIZE] ^= byte;
            output[(i + 5) % SIZE] ^= output[(i + 3) % SIZE] & byte;

            output[(i + 17) % SIZE] &= byte;
            output[(i + 8) % SIZE] |= byte ^ output[i % SIZE] & !output[(i + 18) % SIZE];

            if i % SIZE == 0 {
                output[(i + 12) % SIZE] = output[(i + 65) % SIZE].rotate_right(7);
            } else if i % SIZE <= SIZE / 2 {
                output[i % SIZE] = u64::wrapping_add(
                    u64::wrapping_add(
                        output[(i + 3) % SIZE],
                        output[(i + 3) % SIZE] | output[(i + 1) % SIZE],
                    ),
                    output[i % SIZE],
                );
                output[(i + 213) % SIZE] = !output[(i + 26) % SIZE];
                output[(i + 46) % SIZE] = output[(i + 72) % SIZE].rotate_left(3);
            } else if i % SIZE >= SIZE / 2 {
                output[i % SIZE] &= 0x53;
                output[(i + 23) % SIZE] = output[(i + 75) % SIZE].rotate_left(5);
            }
            output[i % SIZE] = output[i % SIZE].rotate_left(55);
        }
    }
    //println!("output: {:?}",output);
    [
        output[0].to_be_bytes().to_vec(),
        output[1].to_be_bytes().to_vec(),
        output[2].to_be_bytes().to_vec(),
        output[3].to_be_bytes().to_vec(),
    ]
    .concat()
}
