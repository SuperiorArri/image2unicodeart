
use image2unicodeart::{generate_image, ProgramError, ProgramParameters};
use clap::Parser;

#[derive(Parser, Debug)]
#[command(about = "Tool for converting images to Unicode art.")]
struct Args {
    #[clap(index = 1)]
    #[arg(help="Input file path or URL")]
    input: String,

    #[arg(short, long, help="Output file path")]
    output: Option<String>,

    #[arg(short, long, help="Output width (number of symbols)")]
    width: Option<u32>,

    #[arg(short, long, default_value_t = 0.5, help="Width/height of symbols")]
    symbol_aspect_ratio: f32,

    #[arg(short, long, default_value_t=String::from(" ░▒▓█"))]
    charset: String,
}

fn main() {
    let args = Args::parse();

    let output_path_opt = args.output.as_ref().map(|x| x.as_ref());

    let res = generate_image(&ProgramParameters {
        input_path: &args.input,
        output_path: output_path_opt,
        output_width: args.width,
        symbol_aspect_ratio: args.symbol_aspect_ratio,
        charset: &args.charset,
    });

    match res {
        Ok(_) => {}
        Err(err) => match err {
            ProgramError::InvalidInputPath => {
                println!("Failed to open: {}", args.input);
            }
            ProgramError::FailedToDecodeInput => {
                println!("Failed to decode input image!");
            }
            ProgramError::FailedToWriteToOutput => {
                println!("Failed to save output to: {}", args.output.unwrap());
            }
            ProgramError::FailedToDownload => {
                println!("Failed to download: {}", args.input);
            },
            ProgramError::DownloadInvalid => {
                println!("Invalid source: {}", args.input);
            },
        },
    }
}
