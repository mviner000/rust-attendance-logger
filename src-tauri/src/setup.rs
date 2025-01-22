// src/setup.rs
use std::thread;
use std::time::Duration;
use anyhow::{Result, anyhow};
use std::path::Path;
use tauri_plugin_shell::ShellExt;
use tauri::Manager;

pub struct SystemSetup;

impl SystemSetup {
    pub async fn check_docker(app: &tauri::AppHandle) -> bool {
        app.shell().command("docker")
            .args(["--version"])
            .output()
            .await
            .is_ok()
    }

    pub async fn download_docker_installer() -> Result<String> {
        let url = "https://desktop.docker.com/win/main/amd64/Docker%20Desktop%20Installer.exe";
        let installer_path = std::env::temp_dir().join("DockerDesktopInstaller.exe");
        
        println!("Downloading Docker Desktop installer...");
        
        let response = reqwest::get(url).await?;
        if !response.status().is_success() {
            return Err(anyhow!("Failed to download Docker installer: HTTP {}", response.status()));
        }
        
        let bytes = response.bytes().await?;
        tokio::fs::write(&installer_path, bytes).await?;
        
        Ok(installer_path.to_string_lossy().into_owned())
    }

    pub async fn install_docker(app: &tauri::AppHandle) -> Result<()> {
        let installer_path = Self::download_docker_installer().await?;
        
        println!("Creating installation script...");
        
        // Create a more robust PowerShell installation script
        // Note: We use a literal $env:ProgramFiles without trying to format it
        let install_script = format!(
            r#"
            $ErrorActionPreference = 'Stop'
            $installerPath = '{}'
    
            function Wait-DockerService {{
                $retries = 0
                $maxRetries = 12
                
                Write-Host "Waiting for Docker service to start..."
                do {{
                    $service = Get-Service -Name "com.docker.service" -ErrorAction SilentlyContinue
                    if ($service -and $service.Status -eq 'Running') {{
                        Write-Host "Docker service is running."
                        return $true
                    }}
                    Start-Sleep -Seconds 10
                    $retries++
                    Write-Host "Waiting... Attempt $retries of $maxRetries"
                }} while ($retries -lt $maxRetries)
                
                return $false
            }}
    
            try {{
                # Check if Docker Desktop is already installed
                $installed = Get-WmiObject -Class Win32_Product | Where-Object {{ $_.Name -like "*Docker Desktop*" }}
                if ($installed) {{
                    Write-Host "Docker Desktop is already installed. Attempting to repair/update..."
                    $process = Start-Process -FilePath $installerPath -ArgumentList "uninstall --quiet" -Wait -PassThru -Verb RunAs
                    Start-Sleep -Seconds 10
                }}
    
                # Install Docker Desktop
                Write-Host "Installing Docker Desktop..."
                $process = Start-Process -FilePath $installerPath -ArgumentList "install --quiet" -Wait -PassThru -Verb RunAs
                
                if ($process.ExitCode -ne 0) {{
                    throw "Installation failed with exit code $($process.ExitCode)"
                }}
    
                Write-Host "Installation completed. Starting Docker..."
                Start-Sleep -Seconds 10
    
                # Start Docker Desktop
                $dockerPath = "$env:ProgramFiles\Docker\Docker\Docker Desktop.exe"
                Start-Process -FilePath $dockerPath
                
                if (Wait-DockerService) {{
                    Write-Host "Docker Desktop installation and startup successful."
                    exit 0
                }} else {{
                    throw "Docker service failed to start after installation."
                }}
            }} catch {{
                Write-Error "Installation failed: $_"
                exit 1
            }} finally {{
                # Cleanup
                if (Test-Path $installerPath) {{
                    Remove-Item -Force $installerPath
                }}
            }}
            "#,
            installer_path
        );
    
        let script_path = std::env::temp_dir().join("docker_install.ps1");
        tokio::fs::write(&script_path, install_script).await?;
    
        println!("Executing installation script...");
        
        let output = app.shell().command("powershell")
            .args([
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-File",
                &script_path.to_string_lossy(),
            ])
            .output()
            .await?;
    
        // Cleanup script
        let _ = tokio::fs::remove_file(script_path).await;
    
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow!("Installation failed: {}", error));
        }
    
        // Verify Docker is running
        for i in 0..6 {
            if Self::check_docker(app).await {
                println!("Docker is now available!");
                return Ok(());
            }
            if i < 5 {
                println!("Waiting for Docker to become available... ({}/5)", i + 1);
                thread::sleep(Duration::from_secs(10));
            }
        }
    
        Err(anyhow!("Docker installation completed but Docker is not responding. Please start Docker Desktop manually."))
    }

    pub async fn check_mysql_container(app: &tauri::AppHandle) -> bool {
        let output = app.shell().command("docker")
            .args(["ps", "-a", "--filter", "name=mysql", "--format", "{{.Names}}"])
            .output()
            .await;
            
        match output {
            Ok(output) => String::from_utf8_lossy(&output.stdout).contains("mysql"),
            Err(_) => false,
        }
    }

    pub async fn start_mysql_container(app: &tauri::AppHandle) -> Result<()> {
        if !Self::check_mysql_container(app).await {
            println!("Creating new MySQL container...");
            
            let output = app.shell().command("docker")
                .args([
                    "run",
                    "-d",
                    "--name", "mysql",
                    "-e", "MYSQL_ROOT_PASSWORD=password",
                    "-e", "MYSQL_DATABASE=app_db",
                    "-p", "3306:3306",
                    "mysql:8.0"
                ])
                .output()
                .await?;
    
            if !output.status.success() {
                return Err(anyhow!("Failed to create MySQL container"));
            }
            println!("✓ MySQL container created successfully");
        } else {
            println!("Starting existing MySQL container...");
            
            let output = app.shell().command("docker")
                .args(["start", "mysql"])
                .output()
                .await?;
    
            if !output.status.success() {
                return Err(anyhow!("Failed to start MySQL container"));
            }
            println!("✓ Existing MySQL container started successfully");
        }
    
        println!("Waiting for MySQL to initialize...");
        thread::sleep(Duration::from_secs(15));
        println!("✓ MySQL initialization complete");
        
        Ok(())
    }
    

    pub async fn setup_system(app: &tauri::AppHandle) -> Result<()> {
        println!("Starting system setup...");

        // Check Windows version compatibility
        if let Ok(is_compatible) = Self::check_windows_version(app).await {
            if !is_compatible {
                return Err(anyhow!("Docker Desktop requires Windows 10/11 Pro, Enterprise, or Education"));
            }
        }

        // Check and install Docker if needed
        if !Self::check_docker(app).await {
            println!("Docker not found. Installing Docker Desktop...");
            Self::install_docker(app).await?;
        } else {
            println!("✓ Docker Desktop is already installed");
        }

        // Verify Docker is running
        if let Ok(running) = Self::is_docker_running(app).await {
            if !running {
                return Err(anyhow!("Docker is installed but not running. Please start Docker Desktop."));
            }
            println!("✓ Docker service is running");

            // Setup MySQL container
            Self::start_mysql_container(app).await?;
        }

        println!("✓ System setup completed successfully");
        Ok(())
    }
    
    
    async fn is_docker_running(app: &tauri::AppHandle) -> Result<bool> {
        let output = app.shell().command("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-Service 'com.docker.service' | Select-Object -ExpandProperty Status"
            ])
            .output()
            .await?;

        let status = String::from_utf8_lossy(&output.stdout).trim().to_lowercase();
        Ok(status == "running")
    }

    pub async fn check_windows_version(app: &tauri::AppHandle) -> Result<bool> {
        let output = app.shell().command("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-CimInstance -ClassName Win32_OperatingSystem | Select-Object Caption,OperatingSystemSKU"
            ])
            .output()
            .await?;

        let output_str = String::from_utf8_lossy(&output.stdout).to_lowercase();
        
        Ok(output_str.contains("pro") || 
           output_str.contains("enterprise") || 
           output_str.contains("education"))
    }
}