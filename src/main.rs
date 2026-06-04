use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use windows::Win32::Devices::Display::{
    DISPLAYCONFIG_MODE_INFO, DISPLAYCONFIG_PATH_INFO, GetDisplayConfigBufferSizes,
    QDC_ONLY_ACTIVE_PATHS, QueryDisplayConfig, SDC_APPLY, SDC_USE_SUPPLIED_DISPLAY_CONFIG,
    SetDisplayConfig,
};
use windows::Win32::Foundation::WIN32_ERROR;
use windows::core::Result;

#[derive(Parser, Debug)]
#[command(name = "display-switcher", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// 現在のディスプレイ状態をプロファイルとして保存する
    Save {
        /// 保存するプロファイル名 (例: all-on, single)
        name: String,
    },
    /// 保存したプロファイルを適用（復元）する
    Apply {
        /// 適用するプロファイル名
        name: String,
    },
}

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Commands::Save { name } => {
            println!("現在のディスプレイ構成を取得中...");
            let (paths, modes) = get_active_displays()?;

            match save_profile(&name, &paths, &modes) {
                Ok(_) => {
                    println!("✅ プロファイル '{}' を保存しました！", name);
                    println!(
                        "📁 profiles フォルダ内に {}.bat が生成されました。これを実行するだけで切り替え可能です。",
                        name
                    );
                }
                Err(e) => println!("❌ 保存に失敗しました: {}", e),
            }
        }
        Commands::Apply { name } => {
            println!("プロファイル '{}' を読み込んでいます...", name);
            match load_profile(&name) {
                Ok((paths, modes)) => {
                    println!("構成を適用中...");
                    if let Err(e) = apply_config(&paths, &modes) {
                        println!("❌ 適用に失敗しました: {}", e);
                    } else {
                        println!(
                            "✅ プロファイル '{}' を適用しました！画面が切り替わります。",
                            name
                        );
                    }
                }
                Err(e) => println!("❌ プロファイルの読み込みに失敗しました: {}", e),
            }
        }
    }

    Ok(())
}

fn get_active_displays() -> Result<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    unsafe {
        let mut path_count = 0;
        let mut mode_count = 0;

        GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
            .ok()?;

        let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); path_count as usize];
        let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); mode_count as usize];

        QueryDisplayConfig(
            QDC_ONLY_ACTIVE_PATHS,
            &mut path_count,
            paths.as_mut_ptr(),
            &mut mode_count,
            modes.as_mut_ptr(),
            None,
        )
        .ok()?;

        paths.truncate(path_count as usize);
        modes.truncate(mode_count as usize);

        Ok((paths, modes))
    }
}

fn apply_config(
    paths: &[DISPLAYCONFIG_PATH_INFO],
    modes: &[DISPLAYCONFIG_MODE_INFO],
) -> Result<()> {
    unsafe {
        let status = SetDisplayConfig(
            Some(paths),
            Some(modes),
            SDC_APPLY | SDC_USE_SUPPLIED_DISPLAY_CONFIG,
        );
        WIN32_ERROR(status as u32).ok()?;
        Ok(())
    }
}

// =====================================================================
// ファイル保存・読み込み処理 (ディレクトリ作成と.bat生成を追加)
// =====================================================================

fn save_profile(
    name: &str,
    paths: &[DISPLAYCONFIG_PATH_INFO],
    modes: &[DISPLAYCONFIG_MODE_INFO],
) -> std::io::Result<()> {
    // 1. profiles ディレクトリがなければ作成する
    let dir = Path::new("profiles");
    if !dir.exists() {
        fs::create_dir_all(dir)?;
    }

    // 2. .dat ファイルを profiles ディレクトリ内に保存
    let dat_filename = dir.join(format!("{}.dat", name));
    let mut file = File::create(&dat_filename)?;

    let paths_len = paths.len() as u64;
    let modes_len = modes.len() as u64;
    file.write_all(&paths_len.to_le_bytes())?;
    file.write_all(&modes_len.to_le_bytes())?;

    unsafe {
        let paths_bytes = std::slice::from_raw_parts(
            paths.as_ptr() as *const u8,
            paths.len() * std::mem::size_of::<DISPLAYCONFIG_PATH_INFO>(),
        );
        file.write_all(paths_bytes)?;

        let modes_bytes = std::slice::from_raw_parts(
            modes.as_ptr() as *const u8,
            modes.len() * std::mem::size_of::<DISPLAYCONFIG_MODE_INFO>(),
        );
        file.write_all(modes_bytes)?;
    }

    // 3. .bat ファイルを profiles ディレクトリ内に生成
    let bat_filename = dir.join(format!("{}.bat", name));
    let mut bat_file = File::create(&bat_filename)?;

    // バッチファイルの中身
    // %~dp0 は「このバッチファイルがあるディレクトリ」を指す
    // cd /d "%~dp0.." で1つ上の階層（exeがある場所）に移動してからコマンドを実行する
    let bat_content = format!(
        "@echo off\n\
        cd /d \"%~dp0..\"\n\
        monitor-switcher.exe apply {}\n",
        name
    );

    bat_file.write_all(bat_content.as_bytes())?;

    Ok(())
}

fn load_profile(
    name: &str,
) -> std::io::Result<(Vec<DISPLAYCONFIG_PATH_INFO>, Vec<DISPLAYCONFIG_MODE_INFO>)> {
    // 読み込む時も profiles ディレクトリの中から探すように変更
    let filename = format!("profiles/{}.dat", name);
    let mut file = File::open(filename)?;

    let mut len_buf = [0u8; 8];

    file.read_exact(&mut len_buf)?;
    let paths_len = u64::from_le_bytes(len_buf) as usize;

    file.read_exact(&mut len_buf)?;
    let modes_len = u64::from_le_bytes(len_buf) as usize;

    let mut paths = vec![DISPLAYCONFIG_PATH_INFO::default(); paths_len];
    let mut modes = vec![DISPLAYCONFIG_MODE_INFO::default(); modes_len];

    unsafe {
        let paths_bytes = std::slice::from_raw_parts_mut(
            paths.as_mut_ptr() as *mut u8,
            paths_len * std::mem::size_of::<DISPLAYCONFIG_PATH_INFO>(),
        );
        file.read_exact(paths_bytes)?;

        let modes_bytes = std::slice::from_raw_parts_mut(
            modes.as_mut_ptr() as *mut u8,
            modes_len * std::mem::size_of::<DISPLAYCONFIG_MODE_INFO>(),
        );
        file.read_exact(modes_bytes)?;
    }

    Ok((paths, modes))
}
