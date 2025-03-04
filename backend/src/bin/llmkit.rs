use anyhow::{Context, Result};
use sqlx::migrate::Migrator;
use std::{
    env, fs,
    path::Path,
    process::{Child, Command, Stdio},
    str::FromStr,
};

// Re-export the static MIGRATOR from the db module
static MIGRATOR: Migrator = sqlx::migrate!();

#[tokio::main]
async fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(());
    }

    match args[1].as_str() {
        "start" => start().await?,
        "help" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", args[1]);
            print_usage();
        }
    }

    Ok(())
}

fn print_usage() {
    println!("LLMKit - A toolkit for LLM applications");
    println!();
    println!("USAGE:");
    println!("    llmkit COMMAND");
    println!();
    println!("COMMANDS:");
    println!("    start       Start the LLMKit application (backend and frontend)");
    println!("    help        Print this help message");
}

/// Find project root directory
fn find_project_root() -> Result<std::path::PathBuf> {
    let mut current_dir = env::current_dir()?;

    // Keep going up until we find a directory with both "backend" and "ui" subdirectories
    loop {
        // Check if this is our project root
        let backend_dir = current_dir.join("backend");
        let ui_dir = current_dir.join("ui");

        if backend_dir.is_dir() && ui_dir.is_dir() {
            return Ok(current_dir);
        }

        // Go up one directory
        if !current_dir.pop() {
            // We've reached the root of the filesystem without finding our project
            return Err(anyhow::anyhow!("Could not find project root directory"));
        }
    }
}

/// Generate a secure random JWT token
fn generate_jwt_secret() -> String {
    use rand::{thread_rng, Rng};
    const CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()-_=+[]{}|;:,.<>?";

    let mut rng = thread_rng();
    let secret: String = (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    secret
}

/// Setup environment variables and .env file if needed
fn setup_env(project_root: &Path) -> Result<()> {
    let backend_dir = project_root.join("backend");
    let env_file = backend_dir.join(".env");
    let env_example = project_root.join(".env.example");

    // Create database directory and path
    let db_filename = "llmkit.db";
    let db_path = backend_dir.join(db_filename);
    let abs_db_path = fs::canonicalize(&backend_dir)?.join(db_filename);
    let database_url = format!("sqlite:{}", abs_db_path.display());

    // If .env doesn't exist but .env.example does, create .env from example
    if !env_file.exists() && env_example.exists() {
        println!("No .env file found in backend directory. Creating one from .env.example...");

        // Read the example file
        let example_content = fs::read_to_string(&env_example)?;

        // Generate values for required fields
        let jwt_secret = generate_jwt_secret();

        // Replace placeholder values with generated ones
        let mut env_content = example_content
            .replace("JWT_SECRET=", &format!("JWT_SECRET={}", jwt_secret))
            .replace("DATABASE_URL=/path/to/db/here", &format!("DATABASE_URL={}", database_url));

        // Write the new .env file
        fs::write(&env_file, env_content)?;

        println!("Created .env file with generated JWT secret and database path");
    }

    // Load environment variables
    if let Ok(path) = env::var("DOTENV_PATH") {
        dotenv::from_path(path).ok();
    } else {
        dotenv::from_path(&env_file).ok();
    }

    // Check if OPENROUTER_API_KEY is missing
    if env::var("OPENROUTER_API_KEY").unwrap_or_default().trim().is_empty() {
        // Print warning in red
        eprintln!(
            "\x1b[1;31mWARNING: OPENROUTER_API_KEY is not set in your .env file!\x1b[0m"
        );
        eprintln!(
            "\x1b[1;31mLLMKit will not function properly without a valid API key.\x1b[0m"
        );
        eprintln!(
            "\x1b[1;31mPlease set OPENROUTER_API_KEY in your .env file and restart.\x1b[0m"
        );
        eprintln!();
    }

    Ok(())
}

/// Start the LLMKit application
async fn start() -> Result<()> {
    println!("Starting LLMKit...");

    // Find the project root directory
    let project_root = find_project_root()?;
    println!("Project root: {}", project_root.display());

    // Change to project root directory
    env::set_current_dir(&project_root)?;

    // Setup environment variables
    setup_env(&project_root)?;

    // Setup database
    setup_database().await?;

    // Give the frontend a chance to install dependencies if needed
    let _ = check_frontend_dependencies(&project_root);

    // Start backend and frontend concurrently
    println!("\n🚀 Starting services...");
    let mut backend_process = start_backend(&project_root)?;

    // Wait a moment for the backend to start properly
    println!("⏳ Waiting for backend to initialize...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Start frontend after backend is ready
    let mut frontend_process = match start_frontend(&project_root) {
        Ok(process) => process,
        Err(e) => {
            eprintln!("❌ Error starting frontend: {}", e);
            // Kill the backend if frontend fails
            let _ = backend_process.kill();
            return Err(e);
        }
    };

    println!("\n✅ LLMKit is now running:");
    println!("  🔹 Backend: http://localhost:8000");
    println!("  🔹 Frontend: http://localhost:3000");
    println!();
    println!("Press Ctrl+C to stop...");

    // Monitor both processes in separate tokio tasks
    let backend_pid = backend_process.id();
    let frontend_pid = frontend_process.id();

    // Exit if either process dies
    tokio::spawn(async move {
        let _ = backend_process.wait();
        println!("⚠️ Backend process terminated unexpectedly");
        std::process::exit(1);
    });

    tokio::spawn(async move {
        let _ = frontend_process.wait();
        println!("⚠️ Frontend process terminated unexpectedly");
        std::process::exit(1);
    });

    // Wait for user to press Ctrl+C
    match tokio::signal::ctrl_c().await {
        Ok(()) => {
            println!("\n🛑 Shutting down...");

            // Stop processes
            println!("Stopping frontend...");
            if let Err(e) = stop_process(frontend_pid) {
                eprintln!("Warning: Failed to stop frontend process: {}", e);
            }

            println!("Stopping backend...");
            if let Err(e) = stop_process(backend_pid) {
                eprintln!("Warning: Failed to stop backend process: {}", e);
            }

            println!("🏁 LLMKit stopped.");
        }
        Err(e) => eprintln!("Error waiting for Ctrl+C: {}", e),
    }

    Ok(())
}

/// Stop a process more gracefully than just kill
fn stop_process(pid: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Command::new("kill").arg(pid.to_string()).spawn()?.wait()?;
    }

    #[cfg(windows)]
    {
        Command::new("taskkill")
            .args(&["/PID", &pid.to_string(), "/F"])
            .spawn()?
            .wait()?;
    }

    Ok(())
}

/// Setup SQLite database and run migrations
async fn setup_database() -> Result<()> {
    let project_root = find_project_root()?;

    // Create a full absolute path to the database in the backend directory
    let db_filename = "llmkit.db";
    let db_path = project_root.join("backend").join(db_filename);
    let abs_db_path = fs::canonicalize(project_root.join("backend"))?.join(db_filename);

    // Generate the database URL using the absolute path
    let database_url = format!("sqlite:{}", db_path.display());

    println!("Database path: {}", db_path.display());

    // Check if permissions may be an issue
    let backend_dir = db_path.parent().unwrap();
    if !backend_dir.exists() {
        fs::create_dir_all(backend_dir)?;
    }

    // Set folder permissions to ensure we can write to it
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = fs::metadata(backend_dir)?;
        let mut perms = metadata.permissions();
        perms.set_mode(0o755); // rwxr-xr-x
        fs::set_permissions(backend_dir, perms)?;
    }

    // Create an empty database file with permissions
    if !db_path.exists() {
        println!("Creating new database file...");
        let file = fs::File::create(&db_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o644); // rw-r--r--
            fs::set_permissions(&db_path, perms)?;
        }

        println!("Database file created at: {}", db_path.display());
    } else {
        println!("Database file already exists");

        // Check if file is readable/writable
        let file = fs::OpenOptions::new().read(true).write(true).open(&db_path);

        if let Err(e) = file {
            eprintln!(
                "Warning: Database file exists but might not be accessible: {}",
                e
            );

            // Try to fix permissions
            #[cfg(unix)]
            {
                println!("Attempting to fix database file permissions...");
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&db_path)?.permissions();
                perms.set_mode(0o644); // rw-r--r--
                fs::set_permissions(&db_path, perms)?;
            }
        }
    }

    // Set the DATABASE_URL environment variable using the absolute path
    let abs_database_url = format!("sqlite:{}", abs_db_path.display());
    println!("Setting DATABASE_URL={}", abs_database_url);
    env::set_var("DATABASE_URL", &abs_database_url);

    // Run migrations directly using native SQLite commands
    // This approach is more reliable than using SQLx migrations in some cases
    if !run_migrations_manually(&project_root, &db_path)? {
        // If manual migrations fail, fall back to SQLx migrations
        println!("Falling back to SQLx migrations...");

        // Run migrations
        println!("Running database migrations via SQLx...");
        let pool = sqlx::SqlitePool::connect_with(
            sqlx::sqlite::SqliteConnectOptions::from_str(&abs_database_url)?
                .create_if_missing(true)
                .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
                .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
                .foreign_keys(true),
        )
        .await
        .context("Failed to connect to database")?;

        // Retry migrations if they fail the first time
        let mut retries = 3;
        let mut last_error = None;

        while retries > 0 {
            match MIGRATOR.run(&pool).await {
                Ok(_) => {
                    println!("Database setup completed successfully");
                    return Ok(());
                }
                Err(e) => {
                    eprintln!("Migration attempt failed: {}", e);
                    last_error = Some(e);
                    retries -= 1;

                    if retries > 0 {
                        println!("Retrying migration... ({} attempts left)", retries);
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                    }
                }
            }
        }

        if let Some(e) = last_error {
            return Err(anyhow::anyhow!(
                "Failed to run migrations after multiple attempts: {}",
                e
            ));
        }
    }

    println!("Database setup completed successfully");
    Ok(())
}

/// Run migrations manually using the sqlite3 command-line tool
fn run_migrations_manually(project_root: &Path, db_path: &Path) -> Result<bool> {
    println!("Attempting to run migrations manually using sqlite3...");

    // Check if sqlite3 command is available
    let sqlite_check = Command::new("sqlite3").arg("--version").output();

    if let Err(_) = sqlite_check {
        println!("sqlite3 command-line tool not found, skipping manual migrations");
        return Ok(false);
    }

    // Get list of migration files
    let migrations_dir = project_root.join("backend").join("migrations");
    if !migrations_dir.exists() {
        println!(
            "Migrations directory not found: {}",
            migrations_dir.display()
        );
        return Ok(false);
    }

    let mut migration_files = Vec::new();
    for entry in fs::read_dir(&migrations_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "sql") {
            migration_files.push(path);
        }
    }

    // Sort migration files by name (which should have timestamp prefixes)
    migration_files.sort();

    if migration_files.is_empty() {
        println!("No migration files found in {}", migrations_dir.display());
        return Ok(false);
    }

    // Run each migration file
    for migration_file in &migration_files {
        println!(
            "Running migration: {}",
            migration_file.file_name().unwrap().to_string_lossy()
        );

        let status = Command::new("sqlite3")
            .arg(db_path)
            .arg(".read")
            .arg(migration_file)
            .status()?;

        if !status.success() {
            eprintln!("Failed to run migration: {}", migration_file.display());
            return Ok(false);
        }
    }

    println!("Manual migrations completed successfully");
    return Ok(true);
}

/// Extract database path from a SQLite connection URL
fn extract_db_path_from_url(url: &str) -> Result<String> {
    if url.starts_with("sqlite:") {
        Ok(url.trim_start_matches("sqlite:").to_string())
    } else {
        anyhow::bail!("Not a valid SQLite database URL: {}", url)
    }
}

// Extension trait for Command to conditionally set environment variables
trait CommandExt {
    fn env_if_exists(&mut self, key: &str) -> &mut Self;
}

impl CommandExt for Command {
    fn env_if_exists(&mut self, key: &str) -> &mut Self {
        if let Ok(value) = env::var(key) {
            self.env(key, value);
        }
        self
    }
}

/// Check and install frontend dependencies if needed
fn check_frontend_dependencies(project_root: &Path) -> Result<()> {
    let ui_dir = project_root.join("ui");
    let node_modules = ui_dir.join("node_modules");

    // If node_modules exists, assume dependencies are installed
    if node_modules.exists() && node_modules.is_dir() {
        return Ok(());
    }

    println!("Frontend dependencies not found. Installing...");

    // Detect which package manager to use
    let cmd = if Path::new("/usr/bin/bun").exists() || Path::new("/usr/local/bin/bun").exists() {
        "bun"
    } else if Path::new("/usr/bin/yarn").exists() || Path::new("/usr/local/bin/yarn").exists() {
        "yarn"
    } else if Path::new("/usr/bin/pnpm").exists() || Path::new("/usr/local/bin/pnpm").exists() {
        "pnpm"
    } else {
        "npm"
    };

    println!("Using {} to install dependencies...", cmd);

    let status = Command::new(cmd)
        .arg("install")
        .current_dir(&ui_dir)
        .status()
        .context("Failed to install frontend dependencies")?;

    if !status.success() {
        eprintln!("Warning: Frontend dependency installation may have failed");
    } else {
        println!("Frontend dependencies installed successfully");
    }

    Ok(())
}

/// Starts the backend server
fn start_backend(project_root: &Path) -> Result<Child> {
    println!("Starting backend server...");

    // Get the DATABASE_URL from environment
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL should be set by setup_database");

    // Make sure JWT_SECRET is set
    if env::var("JWT_SECRET").is_err() {
        println!("Generating a JWT_SECRET for the backend...");
        env::set_var("JWT_SECRET", generate_jwt_secret());
    }

    // Create a pipe for capturing stderr to filter output
    let (stderr_read, stderr_write) = match os_pipe::pipe() {
        Ok((read, write)) => (read, write),
        Err(e) => {
            eprintln!("Warning: Failed to create pipe for backend output: {}", e);
            return start_backend_with_inherited_io(project_root, db_url);
        }
    };

    // Pass all environment variables explicitly
    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "backend", "--quiet"])
        .current_dir(project_root.join("backend"))
        .env("DATABASE_URL", &db_url)
        .env(
            "JWT_SECRET",
            env::var("JWT_SECRET").unwrap_or_else(|_| generate_jwt_secret()),
        )
        .env(
            "RUST_LOG",
            env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .stdout(Stdio::null()) // Discard stdout
        .stderr(stderr_write); // Capture stderr for filtering

    let child = match cmd.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!("Failed to start backend with custom output handling: {}", e);
            return start_backend_with_inherited_io(project_root, db_url);
        }
    };

    // Start a thread to monitor stderr and only print error messages
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stderr_read);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    // Only print lines that look like errors or warnings
                    if line.contains("ERROR")
                        || line.contains("error")
                        || line.contains("WARN")
                        || line.contains("warning")
                        || line.contains("panic")
                        || line.contains("PANIC")
                        || line.contains("exception")
                    {
                        eprintln!("[Backend] {}", line);
                    }
                }
                Err(_) => break,
            }
        }
    });

    Ok(child)
}

/// Fallback method for starting backend with standard IO handling
fn start_backend_with_inherited_io(project_root: &Path, db_url: String) -> Result<Child> {
    println!("Falling back to standard output mode...");

    let mut cmd = Command::new("cargo");
    cmd.args(["run", "--bin", "backend", "--quiet"])
        .current_dir(project_root.join("backend"))
        .env("DATABASE_URL", db_url)
        .env(
            "JWT_SECRET",
            env::var("JWT_SECRET").unwrap_or_else(|_| generate_jwt_secret()),
        )
        .env(
            "RUST_LOG",
            env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        )
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let child = cmd.spawn().context("Failed to start backend server")?;

    Ok(child)
}

/// Starts the frontend development server
fn start_frontend(project_root: &Path) -> Result<Child> {
    println!("Starting frontend server...");

    let ui_dir = project_root.join("ui");

    // Detect which package manager is available
    let (cmd, args) = if ui_dir.join("bun.lock").exists() || ui_dir.join("bun.lockb").exists() {
        ("bun", vec!["run", "dev", "--silent"])
    } else if ui_dir.join("yarn.lock").exists() {
        ("yarn", vec!["dev", "--silent"])
    } else if ui_dir.join("pnpm-lock.yaml").exists() {
        ("pnpm", vec!["dev", "--silent"])
    } else {
        ("npm", vec!["run", "dev", "--silent"])
    };

    // Create pipes for capturing and filtering output
    let (stdout_read, stdout_write) = match os_pipe::pipe() {
        Ok((read, write)) => (read, write),
        Err(e) => {
            eprintln!("Warning: Failed to create pipe for frontend output: {}", e);
            return start_frontend_with_inherited_io(project_root);
        }
    };

    let (stderr_read, stderr_write) = match os_pipe::pipe() {
        Ok((read, write)) => (read, write),
        Err(e) => {
            eprintln!("Warning: Failed to create pipe for frontend output: {}", e);
            return start_frontend_with_inherited_io(project_root);
        }
    };

    // Set the environment variable for the frontend to connect to the backend
    let mut process = Command::new(cmd);
    process
        .args(args)
        .current_dir(ui_dir)
        .env("API_BASE_URL", "http://localhost:8000")
        .stdout(stdout_write)
        .stderr(stderr_write);

    let child = match process.spawn() {
        Ok(child) => child,
        Err(e) => {
            eprintln!(
                "Failed to start frontend with custom output handling: {}",
                e
            );
            return start_frontend_with_inherited_io(project_root);
        }
    };

    // Monitor stdout in a separate thread
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stdout_read);

        for line in reader.lines() {
            if let Ok(line) = line {
                // Filter out noise, only show important messages
                if line.contains("error")
                    || line.contains("ERROR")
                    || line.contains("listening on")
                    || line.contains("http://localhost")
                {
                    println!("[Frontend] {}", line);
                }
            }
        }
    });

    // Monitor stderr in a separate thread
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(stderr_read);

        for line in reader.lines() {
            if let Ok(line) = line {
                // Always show errors in stderr
                eprintln!("[Frontend] {}", line);
            }
        }
    });

    Ok(child)
}

/// Fallback method for starting frontend with standard IO handling
fn start_frontend_with_inherited_io(project_root: &Path) -> Result<Child> {
    println!("Falling back to standard output mode for frontend...");

    let ui_dir = project_root.join("ui");

    // Detect which package manager is available
    let (cmd, args) = if ui_dir.join("bun.lock").exists() || ui_dir.join("bun.lockb").exists() {
        ("bun", vec!["run", "dev", "--silent"])
    } else if ui_dir.join("yarn.lock").exists() {
        ("yarn", vec!["dev", "--silent"])
    } else if ui_dir.join("pnpm-lock.yaml").exists() {
        ("pnpm", vec!["dev", "--silent"])
    } else {
        ("npm", vec!["run", "dev", "--silent"])
    };

    // Set the environment variable for the frontend to connect to the backend
    let mut process = Command::new(cmd);
    process
        .args(args)
        .current_dir(ui_dir)
        .env("API_BASE_URL", "http://localhost:8000")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    let child = process.spawn().context("Failed to start frontend server")?;

    Ok(child)
}
