use anyhow::{Context, Result};
use clap::Parser;
use colored::Colorize;
use console::Term;
use fbxcel::low::FbxVersion;
use fbxcel::writer::v7400::binary::{FbxFooter, Writer};
use gltf::Document;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "glb2fbx")]
#[command(author = "Trident_For_U")]
#[command(version = "1.0.0")]
#[command(about = "⚡ The Ultimate GLB to FBX Converter", long_about = None)]
struct Args {
    #[arg(short, long, help = "Input folder containing GLB files")]
    input: PathBuf,

    #[arg(short, long, help = "Output folder for FBX files")]
    output: PathBuf,
}

fn print_banner() {
    let term = Term::stdout();
    let _ = term.clear_screen();
    
    println!();
    println!("{}", "    ╔══════════════════════════════════════════════════════════════════╗".bright_cyan().bold());
    println!("{}", "    ║                                                                  ║".bright_cyan().bold());
    println!("{}", "    ║     ██████╗ ██╗     ██████╗     ██████╗ ███████╗██████╗ ██╗  ██╗ ║".bright_cyan().bold());
    println!("{}", "    ║    ██╔════╝ ██║     ██╔══██╗    ╚════██╗██╔════╝██╔══██╗╚██╗██╔╝ ║".bright_cyan().bold());
    println!("{}", "    ║    ██║  ███╗██║     ██████╔╝     █████╔╝█████╗  ██████╔╝ ╚███╔╝  ║".bright_cyan().bold());
    println!("{}", "    ║    ██║   ██║██║     ██╔══██╗    ██╔═══╝ ██╔══╝  ██╔══██╗ ██╔██╗  ║".bright_cyan().bold());
    println!("{}", "    ║    ╚██████╔╝███████╗██████╔╝    ███████╗██║     ██████╔╝██╔╝ ██╗ ║".bright_cyan().bold());
    println!("{}", "    ║     ╚═════╝ ╚══════╝╚═════╝     ╚══════╝╚═╝     ╚═════╝ ╚═╝  ╚═╝ ║".bright_cyan().bold());
    println!("{}", "    ║                                                                  ║".bright_cyan().bold());
    println!("{}", "    ║              🚀 The Ultimate 3D Model Converter 🚀              ║".bright_magenta().bold());
    println!("{}", "    ║                                                                  ║".bright_cyan().bold());
    println!("{}", "    ║                    Created by: Trident_For_U                     ║".bright_yellow().bold());
    println!("{}", "    ║                         Version 1.0.0                            ║".bright_white());
    println!("{}", "    ║                                                                  ║".bright_cyan().bold());
    println!("{}", "    ╚══════════════════════════════════════════════════════════════════╝".bright_cyan().bold());
    println!();
    
    // Animated loading
    print!("    {}", "Initializing".bright_white().bold());
    for _ in 0..3 {
        std::thread::sleep(std::time::Duration::from_millis(150));
        print!("{}", ".".bright_cyan());
        let _ = std::io::Write::flush(&mut std::io::stdout());
    }
    println!(" {}", "✓".green().bold());
    println!();
}

fn format_file_size(size: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size >= GB {
        format!("{:.2} GB", size as f64 / GB as f64)
    } else if size >= MB {
        format!("{:.2} MB", size as f64 / MB as f64)
    } else if size >= KB {
        format!("{:.2} KB", size as f64 / KB as f64)
    } else {
        format!("{} B", size)
    }
}

fn print_separator(style: &str) {
    match style {
        "thick" => println!("    {}", "═".repeat(66).bright_cyan().bold()),
        "thin" => println!("    {}", "─".repeat(66).bright_black().bold()),
        "double" => println!("    {}", "╬".repeat(66).bright_magenta().bold()),
        _ => println!("    {}", "─".repeat(66).bright_black()),
    }
}

fn main() -> Result<()> {
    print_banner();

    let args = Args::parse();

    // Validate input
    print_separator("thin");
    println!("    {} {}", "📂 INPUT:".bright_blue().bold(), args.input.display().to_string().bright_yellow());
    
    if !args.input.exists() {
        println!("    {} Input folder does not exist!", "❌".red().bold());
        anyhow::bail!("Input folder not found");
    }
    println!("    {} Input validated", "✓".green().bold());
    
    println!("    {} {}", "📁 OUTPUT:".bright_blue().bold(), args.output.display().to_string().bright_yellow());
    fs::create_dir_all(&args.output)
        .context("Failed to create output directory")?;
    println!("    {} Output directory ready", "✓".green().bold());
    
    print_separator("thin");
    println!();

    // Scanning phase with animation
    print!("    {} Scanning for GLB files", "🔍".bright_white().bold());
    let _ = std::io::Write::flush(&mut std::io::stdout());
    
    let scan_start = Instant::now();
    let glb_files: Vec<PathBuf> = WalkDir::new(&args.input)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().is_file() && 
            e.path().extension()
                .map(|ext| ext.eq_ignore_ascii_case("glb"))
                .unwrap_or(false)
        })
        .map(|e| e.path().to_path_buf())
        .collect();
    
    let scan_duration = scan_start.elapsed();
    println!(" {} ({}ms)", "✓".green().bold(), scan_duration.as_millis());
    println!();

    let total_files = glb_files.len();
    
    if total_files == 0 {
        print_separator("thick");
        println!();
        println!("    {} {}", "⚠".yellow().bold(), "No GLB files found in the input directory.".bright_yellow().bold());
        println!();
        print_separator("thick");
        return Ok(());
    }

    // Calculate total input size
    let total_input_size: u64 = glb_files.iter()
        .filter_map(|p| fs::metadata(p).ok())
        .map(|m| m.len())
        .sum();

    // File discovery summary box
    print_separator("thick");
    println!("    {}", "📊 DISCOVERY SUMMARY".bright_white().bold());
    print_separator("thin");
    println!("    {} {} GLB file(s) found", "✓".green().bold(), total_files.to_string().bright_white().bold());
    println!("    {} Total size: {}", "💾".bright_blue(), format_file_size(total_input_size).bright_white().bold());
    println!("    {} Scan time: {}ms", "⚡".bright_yellow(), scan_duration.as_millis().to_string().bright_white().bold());
    print_separator("thick");
    println!();

    // Conversion phase header
    println!("    {}", "🔄 CONVERSION PHASE".bright_magenta().bold());
    print_separator("thin");
    println!();

    // Create fancy progress bar
    let pb = ProgressBar::new(total_files as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("    {spinner:.cyan.bold} [{bar:40.cyan/blue}] {pos}/{len} │ {msg}")
            .unwrap()
            .progress_chars("█▓▒░ ")
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
    );

    let mut converted_count = 0;
    let mut failed_count = 0;
    let mut total_output_size = 0u64;
    let conversion_start = Instant::now();

    for (index, path) in glb_files.iter().enumerate() {
        let file_name = path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        
        let input_size = fs::metadata(path)
            .map(|m| m.len())
            .unwrap_or(0);
        
        pb.set_message(format!("{} {}", 
            format!("[{}/{}]", index + 1, total_files).bright_black().bold(),
            file_name.bright_white().bold()
        ));
        
        let file_start = Instant::now();
        match convert_glb_to_fbx(path, &args.output) {
            Ok(output_path) => {
                let output_size = fs::metadata(&output_path)
                    .map(|m| m.len())
                    .unwrap_or(0);
                total_output_size += output_size;
                
                let duration = file_start.elapsed();
                let output_name = output_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown");
                
                pb.println(format!("    {} {} → {} {} {} {}",
                    "✓".green().bold(),
                    file_name.bright_white(),
                    output_name.bright_cyan(),
                    format!("({})", format_file_size(output_size)).bright_black(),
                    "│".bright_black(),
                    format!("{}ms", duration.as_millis()).bright_yellow()
                ));
                converted_count += 1;
            }
            Err(e) => {
                pb.println(format!("    {} {} {} {}",
                    "✗".red().bold(),
                    file_name.bright_white(),
                    "│".bright_black(),
                    format!("{}", e).red()
                ));
                failed_count += 1;
            }
        }
        
        pb.inc(1);
    }

    pb.finish_and_clear();
    
    let total_duration = conversion_start.elapsed();

    // Final summary with fancy box
    println!();
    print_separator("double");
    println!("    {}", "🎉 CONVERSION COMPLETE 🎉".bright_green().bold());
    print_separator("double");
    println!();
    
    // Stats box
    println!("    {}", "📈 STATISTICS".bright_white().bold());
    print_separator("thin");
    println!("    {} {}  {}", 
        "✓".green().bold(), 
        "Successful:".bright_white(), 
        converted_count.to_string().green().bold()
    );
    
    if failed_count > 0 {
        println!("    {} {}      {}", 
            "✗".red().bold(), 
            "Failed:".bright_white(), 
            failed_count.to_string().red().bold()
        );
    }
    
    println!("    {} {}       {}", 
        "Σ".bright_blue().bold(), 
        "Total:".bright_white(), 
        total_files.to_string().bright_white().bold()
    );
    print_separator("thin");
    println!();
    
    // Performance metrics
    println!("    {}", "⚡ PERFORMANCE".bright_white().bold());
    print_separator("thin");
    println!("    {} {}  {}", 
        "⏱".bright_yellow(), 
        "Duration:".bright_white(), 
        format!("{:.2}s", total_duration.as_secs_f64()).bright_white().bold()
    );
    
    let files_per_sec = total_files as f64 / total_duration.as_secs_f64();
    println!("    {} {}     {}", 
        "🚀".bright_cyan(), 
        "Speed:".bright_white(), 
        format!("{:.2} files/s", files_per_sec).bright_white().bold()
    );
    
    println!("    {} {}  {}", 
        "💾".bright_blue(), 
        "Input Size:".bright_white(), 
        format_file_size(total_input_size).bright_white().bold()
    );
    
    println!("    {} {} {}", 
        "💿".bright_magenta(), 
        "Output Size:".bright_white(), 
        format_file_size(total_output_size).bright_white().bold()
    );
    
    let ratio = if total_input_size > 0 {
        (total_output_size as f64 / total_input_size as f64) * 100.0
    } else {
        0.0
    };
    
    println!("    {} {}      {}", 
        "📊".bright_green(), 
        "Ratio:".bright_white(), 
        format!("{:.1}%", ratio).bright_white().bold()
    );
    print_separator("thin");
    println!();
    
    // Footer
    print_separator("thick");
    println!("    {}", format!("Made with {} by {} │ Thank you for using GLB2FBX!", 
        "❤️".red(),
        "Trident_For_U".bright_yellow().bold()
    ).bright_white());
    print_separator("thick");
    println!();

    Ok(())
}

fn convert_glb_to_fbx(input_path: &Path, output_dir: &Path) -> Result<PathBuf> {
    let file_stem = input_path
        .file_stem()
        .context("Failed to get file stem")?
        .to_str()
        .context("Invalid UTF-8 in filename")?;

    let output_path = output_dir.join(format!("{}.fbx", file_stem));

    let (gltf, buffers, _images) = gltf::import(input_path)
        .context("Failed to load GLB file")?;

    let file = fs::File::create(&output_path)
        .context("Failed to create output file")?;
    let writer_sink = BufWriter::new(file);

    let mut writer = Writer::new(writer_sink, FbxVersion::V7_4)
        .map_err(|e| anyhow::anyhow!("Failed to create FBX writer: {:?}", e))?;

    // Write FBX tree
    write_fbx_tree(&mut writer, &gltf, &buffers)?;

    // Finalize FBX file
    let footer = FbxFooter::default();
    writer.finalize(&footer)
        .map_err(|e| anyhow::anyhow!("Failed to finalize FBX: {:?}", e))?;

    Ok(output_path)
}

fn write_fbx_tree(
    writer: &mut Writer<BufWriter<fs::File>>,
    gltf: &Document,
    buffers: &[gltf::buffer::Data],
) -> Result<()> {
    // Write FBXHeaderExtension node
    {
        let mut attrs = writer.new_node("FBXHeaderExtension")
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
        // Header Version
        attrs.append_i32(1003)
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
        drop(attrs);
        writer.close_node()
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
    }

    // Write GlobalSettings node with proper properties
    {
        writer.new_node("GlobalSettings")
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
        
        // Version
        {
            let mut attrs = writer.new_node("Version")
                .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            attrs.append_i32(1000)
                .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            drop(attrs);
            writer.close_node()
                .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
        }
        
        // Properties70
        {
            writer.new_node("Properties70")
                .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            
            // UpAxis: P: "UpAxis", "int", "Integer", "",1
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("UpAxis")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(1)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // UpAxisSign
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("UpAxisSign")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(1)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // FrontAxis
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("FrontAxis")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(2)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // FrontAxisSign
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("FrontAxisSign")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(1)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // CoordAxis
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("CoordAxis")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(0)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // CoordAxisSign
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("CoordAxisSign")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(1)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // OriginalUpAxis
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("OriginalUpAxis")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(-1)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // OriginalUpAxisSign
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("OriginalUpAxisSign")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("int")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Integer")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i32(1)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // UnitScaleFactor
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("UnitScaleFactor")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("double")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Number")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_f64(1.0)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            // OriginalUnitScaleFactor
            {
                let mut attrs = writer.new_node("P")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("OriginalUnitScaleFactor")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("double")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Number")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_f64(1.0)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
            
            writer.close_node()
                .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?; // Close Properties70
        }
        
        writer.close_node()
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?; // Close GlobalSettings
    }

    // Write Definitions node
    {
        let mesh_count = gltf.meshes().count();
        let node_count = gltf.nodes().count();
        let mut attrs = writer.new_node("Definitions")
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
        attrs.append_i32((mesh_count + node_count) as i32)
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
        drop(attrs);
        writer.close_node()
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
    }

    // Write Objects node
    {
        writer.new_node("Objects")
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;

        // Write geometries (meshes)
        for mesh in gltf.meshes() {
            let mesh_id = (mesh.index() + 1) * 10000;
            let mesh_name = mesh.name().unwrap_or("Mesh").to_string();
            
            let mut all_positions = Vec::new();
            let mut all_indices = Vec::new();
            let mut vertex_offset = 0u32;
            
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));
                
                if let Some(iter) = reader.read_positions() {
                    all_positions.extend(iter);
                }
                
                if let Some(iter) = reader.read_indices() {
                    all_indices.extend(iter.into_u32().map(|i| i + vertex_offset));
                }
                
                vertex_offset = all_positions.len() as u32;
            }
            
            // Start geometry node
            {
                let mut attrs = writer.new_node("Geometry")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i64(mesh_id as i64)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct(&format!("Geometry::{}", mesh_name))
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Mesh")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                
                // Write Vertices child node
                {
                    let mut attrs = writer.new_node("Vertices")
                        .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                    let vertices_flat = all_positions.iter()
                        .flat_map(|v| vec![v[0] as f64, v[1] as f64, v[2] as f64]);
                    attrs.append_arr_f64_from_iter(None, vertices_flat)
                        .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                    drop(attrs);
                    writer.close_node()
                        .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                }
                
                // Write PolygonVertexIndex child node
                {
                    let mut attrs = writer.new_node("PolygonVertexIndex")
                        .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                    let indices_iter = all_indices.iter().enumerate()
                        .map(|(i, &idx)| {
                            if (i + 1) % 3 == 0 {
                                -(idx as i32) - 1
                            } else {
                                idx as i32
                            }
                        });
                    attrs.append_arr_i32_from_iter(None, indices_iter)
                        .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                    drop(attrs);
                    writer.close_node()
                        .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                }
                
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?; // End Geometry
            }
        }
        
        // Write models (nodes)
        for node in gltf.nodes() {
            let node_id = (node.index() + 1) * 20000;
            let node_name = node.name().unwrap_or("Node").to_string();
            
            {
                let mut attrs = writer.new_node("Model")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i64(node_id as i64)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct(&format!("Model::{}", node_name))
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("Mesh")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?; // End Model
            }
        }
        
        writer.close_node()
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?; // End Objects
    }

    // Write Connections node
    {
        writer.new_node("Connections")
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            
        for node in gltf.nodes() {
            if let Some(mesh) = node.mesh() {
                let node_id = (node.index() + 1) * 20000;
                let mesh_id = (mesh.index() + 1) * 10000;
                
                let mut attrs = writer.new_node("C")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_string_direct("OO")
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i64(mesh_id as i64)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                attrs.append_i64(node_id as i64)
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
                drop(attrs);
                writer.close_node()
                    .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?;
            }
        }
        
        writer.close_node()
            .map_err(|e| anyhow::anyhow!("FBX write error: {:?}", e))?; // End Connections
    }

    Ok(())
}
