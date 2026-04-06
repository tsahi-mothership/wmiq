# wmiq

> A modern, concise WMI/CIM command-line tool — the spiritual successor to WMIC.

Microsoft retired `wmic.exe` but left no concise CLI replacement. `wmiq` fills that gap: short aliases, clean output, and full WQL power when you need it.

## Why

- `wmic cpu get name` → 4 words. `Get-CimInstance Win32_Processor | Select-Object Name` → pain.
- WMIC had aliases, an interactive REPL, and worked in batch files. Nothing replaced any of that.
- wmiq does.

## Quick Start

```sh
wmiq cpu                        # sensible defaults — name, cores, speed
wmiq os                         # caption, version, build
wmiq disk                       # model, size, partitions
wmiq cpu name,cores             # pick your columns
wmiq process name,pid,memory    # Win32_Process
```

## Features

### Aliases
Short names that map to WMI classes with sensible default columns:

| Alias | WMI Class | Default Columns |
|-------|-----------|-----------------|
| `cpu` | Win32_Processor | Name, Cores, MaxClockSpeed |
| `os` | Win32_OperatingSystem | Caption, Version, BuildNumber, OSArchitecture |
| `disk` | Win32_DiskDrive | Model, Size, Partitions |
| `mem` | Win32_PhysicalMemory | Manufacturer, Capacity, Speed |
| `nic` | Win32_NetworkAdapterConfiguration | Description, IPAddress, MACAddress |
| `process` | Win32_Process | Name, ProcessId, WorkingSetSize |
| `service` | Win32_Service | Name, DisplayName, State, StartMode |
| `bios` | Win32_BIOS | Manufacturer, SMBIOSBIOSVersion, ReleaseDate |
| `board` | Win32_BaseBoard | Manufacturer, Product, SerialNumber |
| `gpu` | Win32_VideoController | Name, DriverVersion, AdapterRAM |
| `vol` | Win32_Volume | DriveLetter, Label, Capacity, FreeSpace |
| `user` | Win32_UserAccount | Name, FullName, Disabled |
| `hotfix` | Win32_QuickFixEngineering | HotFixID, InstalledOn |
| `startup` | Win32_StartupCommand | Name, Command, Location |
| `share` | Win32_Share | Name, Path, Description |

### Raw WQL Queries
```sh
wmiq -q "SELECT Name, ProcessId FROM Win32_Process WHERE Name LIKE '%chrome%'"
```

### Output Formats
```sh
wmiq cpu                        # table (default)
wmiq cpu -o json                # JSON
wmiq cpu -o csv                 # CSV
wmiq cpu -o list                # key=value pairs (like wmic list full)
```

### Interactive REPL
```sh
wmiq -i                         # enter interactive mode
wmiq> cpu
wmiq> os version,build
wmiq> :q "SELECT * FROM Win32_Processor"
wmiq> :aliases                  # list all aliases
wmiq> :props Win32_Processor    # list all properties of a class
wmiq> :exit
```

### Schema Discovery
```sh
wmiq explore Win32_Processor        # list all properties and methods
wmiq explore Win32_Processor -v     # verbose — types, descriptions, qualifiers
wmiq classes                        # list all classes in root\cimv2
wmiq classes -n root\wmi            # list classes in a different namespace
wmiq namespaces                     # list all WMI namespaces
```

### Event Watching
```sh
wmiq watch process-create           # watch for new processes
wmiq watch service-change           # watch for service state changes
wmiq watch -q "SELECT * FROM __InstanceCreationEvent WITHIN 2 WHERE TargetInstance ISA 'Win32_Process'"
```

### Remote Connections
```sh
wmiq cpu -r server01                # query remote machine
wmiq cpu -r server01 -u admin       # with credentials
```

### WMIC Compatibility Mode
```sh
wmiq compat "wmic os get caption,version"    # parse and execute old WMIC syntax
wmiq migrate script.bat                       # scan batch file, suggest replacements
```

### Batch-Friendly
```sh
# pipe-friendly — no extra headers or decoration
wmiq cpu name --raw | head -1

# exit codes: 0 = results found, 1 = no results, 2 = error
wmiq process name,pid -w "Name='notepad.exe'" && echo found
```

### Security Audit
```sh
wmiq audit                          # scan WMI for persistence, suspicious subscriptions
wmiq audit --namespace root\cimv2   # audit specific namespace
wmiq audit --permissions            # check namespace ACLs
```

## Installation

```sh
# From GitHub releases (single binary, no runtime needed)
# Windows x64
curl -Lo wmiq.exe https://github.com/tsahi-mothership/wmiq/releases/latest/download/wmiq-x86_64-pc-windows-msvc.exe

# Or via cargo
cargo install wmiq

# Or via scoop
scoop install wmiq

# Or via winget
winget install wmiq
```

## Built With

- **Rust** — fast single binary, no runtime
- [`wmi-rs`](https://github.com/ohadravid/wmi-rs) — WMI bindings with serde
- [`clap`](https://github.com/clap-rs/clap) — argument parsing
- [`tabled`](https://github.com/zhiburt/tabled) — terminal table formatting
- [`rustyline`](https://github.com/kkawakam/rustyline) — REPL line editing

## Configuration

wmiq looks for `~/.wmiq/aliases.toml` for custom alias definitions:

```toml
[aliases.myapp]
class = "Win32_Process"
columns = ["Name", "ProcessId", "WorkingSetSize"]
where = "Name LIKE '%myapp%'"
```

## Comparison

| Task | WMIC | PowerShell | wmiq |
|------|------|------------|------|
| CPU name | `wmic cpu get name` | `Get-CimInstance Win32_Processor \| Select Name` | `wmiq cpu name` |
| OS info | `wmic os list brief` | `Get-CimInstance Win32_OperatingSystem \| Format-Table` | `wmiq os` |
| JSON output | ❌ | `\| ConvertTo-Json` | `wmiq os -o json` |
| Interactive | `wmic` (enter) | ❌ | `wmiq -i` |
| Event watch | ❌ | 5+ lines of script | `wmiq watch process-create` |
| Remote | `wmic /node:x` | `New-CimSession` + pipe | `wmiq cpu -r x` |

## License

MIT
