mod sha1;
mod usha;

use sha1::sha1;
use usha::usha0::usha0;

use rand::{thread_rng, Rng};

use indicatif::{ProgressBar, ProgressStyle};

use chrono::prelude::*;

const DATAPOINTS: usize = 100;
const DATABYTES: usize = 256;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Current Timestamp: {:?}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );

    // See: https://datatracker.ietf.org/doc/html/rfc6238#appendix-B for TOTP test
    println!("TOTP-Test:");
    let token = "12345678901234567890";
    let time_secs = vec![
        59,
        1111111109,
        1111111111,
        1234567890,
        2000000000,
        20000000000,
    ];
    for time in time_secs {
        println!(
            "Time: {},UTC: {},T: {},TOTP: {}",
            time,
            seconds_to_pretty(time),
            hex::encode(seconds_to_t(time, 30).to_be_bytes()),
            totp(token.as_bytes(), time, 30, 8)
        );
    }

    println!("\nSHA1 and USHA0 Avalanche effect statistics:");

    // Correct output would be: '2fd4e1c67a2d28fced849ee1bb76e7391b93eb12'
    println!(
        "output: '{}'",
        hex::encode(sha1::sha1(
            "The quick brown fox jumps over the lazy dog".as_bytes()
        ))
    );
    // SHA1
    let progress_bar = ProgressBar::new(DATAPOINTS as u64);
    progress_bar.set_draw_delta(1000);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("SHA1 {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({per_sec}) ({eta})")
        .progress_chars("#>-"));

    // Generate Avalanche effect coeficient:
    let data_points_sha1: Vec<f64> =
        generate_data_points_for_function(sha1::sha1, 160, &progress_bar);
    progress_bar.finish();

    let plot_data_sha1 = plotlib::repr::Histogram::from_slice(
        &data_points_sha1,
        plotlib::repr::HistogramBins::Count(10000),
    );

    println!(
        "SHA1 avg: {:?}",
        data_points_sha1.iter().sum::<f64>() / data_points_sha1.len() as f64
    );

    let view_sha1 = plotlib::view::ContinuousView::new()
        .add(plot_data_sha1)
        .x_label("Bits Changed %")
        .y_label("Frequency");

    plotlib::page::Page::single(&view_sha1).save("sha1.svg")?;

    // USHA0
    let progress_bar = ProgressBar::new(DATAPOINTS as u64);
    progress_bar.set_draw_delta(1000);
    progress_bar.set_style(ProgressStyle::default_bar()
        .template("USHA0 {spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({per_sec}) ({eta})")
        .progress_chars("#>-"));

    // Generate Avalanche effect coeficient:
    let data_points_usha0: Vec<f64> =
        generate_data_points_for_function(usha::usha0::usha0, 256, &progress_bar);
    progress_bar.finish();

    println!(
        "USHA0 avg: {:?}",
        data_points_usha0.iter().sum::<f64>() / data_points_usha0.len() as f64
    );
    println!(
        "USHA0 output: '{}'",
        hex::encode(usha::usha0::usha0(
            "The quick brown fox jumps over the lazy dog".as_bytes()
        ))
    );
    println!(
        "USHA0 output: '{}'",
        hex::encode(usha::usha0::usha0(
            "The quick brown fox jumps over the lazy dog".as_bytes()
        ))
    );

    let plot_data_usha0 = plotlib::repr::Histogram::from_slice(
        &data_points_usha0,
        plotlib::repr::HistogramBins::Count(10000),
    );

    let view_usha0 = plotlib::view::ContinuousView::new()
        .add(plot_data_usha0)
        .x_label("Bits Changed %")
        .y_label("Frequency");

    plotlib::page::Page::single(&view_usha0).save("usha0.svg")?;

    Ok(())
}

fn generate_data_points_for_function<F>(
    function: F,
    output_bits: u32,
    progress_bar: &ProgressBar,
) -> Vec<f64>
where
    F: Fn(&[u8]) -> Vec<u8>,
{
    let mut data_points = vec![];
    for i in 0..DATAPOINTS {
        let mut original = [0u8; DATABYTES];
        thread_rng().fill(&mut original[..]);

        let random_byte = thread_rng().gen_range(0..DATABYTES);
        let random_bit = thread_rng().gen_range(0..8);

        let mut bit_flipped = original;
        let mask = 1 << random_bit;
        bit_flipped[random_byte] ^= mask;

        // https://crypto.stackexchange.com/questions/34269/calculation-of-the-avalanche-effect-coefficient
        let set_bits = bits_set(&generic_size_xor(
            &function(&original),
            &function(&bit_flipped),
        ));

        let coefficent: f64 = set_bits as f64 / output_bits as f64;
        data_points.push(coefficent);
        //println!("{}",coefficent);

        progress_bar.inc(1);
    }
    data_points
}

fn bits_set(bytes: &[u8]) -> u32 {
    let mut set: u32 = 0;
    for byte in bytes {
        set += byte.count_ones();
    }
    set
}

fn generic_size_xor(bits1: &[u8], bits2: &[u8]) -> Vec<u8> {
    let mut output = vec![0u8; bits1.len()];
    for (i, (byte1, byte2)) in bits1.iter().zip(bits2).enumerate() {
        output[i] = byte1 ^ byte2;
        //println!("1: {:08b},2: {:08b}, {:08b}",byte1,byte2,output[i]);
    }
    output
}

const BYTES_OUT: usize = 20;
const BYTES_BLOCK: usize = 64;

fn hmac(key: &[u8], data: &[u8]) -> Vec<u8> {
    // See: https://datatracker.ietf.org/doc/html/rfc2104
    // let ipad: Vec<u8> = vec![0x36; BYTES];
    // let opad: Vec<u8> = vec![0x5C; BYTES];

    // println!("key: '{}'",hex::encode(key));

    let key_padded: Vec<u8> = if key.len() == BYTES_BLOCK {
        (*<&[u8]>::clone(&key)).to_vec()
    } else if key.len() > BYTES_BLOCK {
        let mut x = sha1(key).to_vec();
        x.resize(BYTES_BLOCK, 0x00);
        x
    } else {
        let mut x = key.to_vec();
        x.resize(BYTES_BLOCK, 0x00);
        x
    };

    // println!("key_padded: '{}'",hex::encode(key_padded.clone()));

    let mut key_ipad: Vec<u8> = key_padded.iter().map(|k| k ^ 0x36).collect();

    key_ipad.extend(data);

    // println!("key_ipad: '{}'",hex::encode(key_ipad.clone()));

    let hashed_key_data = sha1(&key_ipad);

    // println!("hashed_key_data: '{}'",hex::encode(hashed_key_data.clone()));

    let mut key_opad: Vec<u8> = key_padded.iter().map(|k| k ^ 0x5C).collect();

    key_opad.extend(hashed_key_data);

    // println!("hashed_key_data: '{}'",hex::encode(key_opad.clone()));

    // println!("hmac: '{}'",hex::encode(sha1(&key_opad)));

    sha1(&key_opad)
}

fn hotp(key: &[u8], counter: u64, digits: u32) -> String {
    // See: https://datatracker.ietf.org/doc/html/rfc4226#section-5
    let hmac = hmac(key, &counter.to_be_bytes());

    let offset = (hmac[hmac.len() - 1] & 0xf) as usize;
    let bin_code: u32 = u32::from_be_bytes([
        hmac[offset] & 0x7f,
        hmac[offset + 1],
        hmac[offset + 2],
        hmac[offset + 3],
    ]);

    let number = bin_code % 10_u32.pow(digits);
    let mut number_string = number.to_string();
    while number_string.len() < digits as usize{
        number_string = "0".to_string() + &number_string;
    }
    number_string
}

fn totp(key: &[u8], seconds: u64, x: u64, digits: u32) -> String {
    hotp(key, seconds_to_t(seconds, x), digits)
}

fn seconds_to_t(seconds: u64, x: u64) -> u64 {
    seconds / x
}
fn current_t(x: u64) -> u64 {
    let current_seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();

    seconds_to_t(current_seconds, x)
}

fn seconds_to_pretty(seconds: u64) -> NaiveDateTime {
    NaiveDateTime::from_timestamp(seconds as i64, 0)
}