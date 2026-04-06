use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AliasEntry {
    pub class: &'static str,
    pub columns: Vec<&'static str>,
    pub filter: Option<&'static str>,
}

pub fn build_alias_registry() -> HashMap<&'static str, AliasEntry> {
    let mut m = HashMap::new();

    m.insert("cpu", AliasEntry {
        class: "Win32_Processor",
        columns: vec!["Name", "NumberOfCores", "MaxClockSpeed"],
        filter: None,
    });
    m.insert("os", AliasEntry {
        class: "Win32_OperatingSystem",
        columns: vec!["Caption", "Version", "BuildNumber", "OSArchitecture"],
        filter: None,
    });
    m.insert("disk", AliasEntry {
        class: "Win32_DiskDrive",
        columns: vec!["Model", "Size", "Partitions"],
        filter: None,
    });
    m.insert("mem", AliasEntry {
        class: "Win32_PhysicalMemory",
        columns: vec!["Manufacturer", "Capacity", "Speed"],
        filter: None,
    });
    m.insert("nic", AliasEntry {
        class: "Win32_NetworkAdapterConfiguration",
        columns: vec!["Description", "IPAddress", "MACAddress"],
        filter: Some("IPEnabled=true"),
    });
    m.insert("process", AliasEntry {
        class: "Win32_Process",
        columns: vec!["Name", "ProcessId", "WorkingSetSize"],
        filter: None,
    });
    m.insert("service", AliasEntry {
        class: "Win32_Service",
        columns: vec!["Name", "DisplayName", "State", "StartMode"],
        filter: None,
    });
    m.insert("bios", AliasEntry {
        class: "Win32_BIOS",
        columns: vec!["Manufacturer", "SMBIOSBIOSVersion", "ReleaseDate"],
        filter: None,
    });
    m.insert("board", AliasEntry {
        class: "Win32_BaseBoard",
        columns: vec!["Manufacturer", "Product", "SerialNumber"],
        filter: None,
    });
    m.insert("gpu", AliasEntry {
        class: "Win32_VideoController",
        columns: vec!["Name", "DriverVersion", "AdapterRAM"],
        filter: None,
    });
    m.insert("vol", AliasEntry {
        class: "Win32_Volume",
        columns: vec!["DriveLetter", "Label", "Capacity", "FreeSpace"],
        filter: None,
    });
    m.insert("user", AliasEntry {
        class: "Win32_UserAccount",
        columns: vec!["Name", "FullName", "Disabled"],
        filter: None,
    });
    m.insert("hotfix", AliasEntry {
        class: "Win32_QuickFixEngineering",
        columns: vec!["HotFixID", "InstalledOn"],
        filter: None,
    });
    m.insert("startup", AliasEntry {
        class: "Win32_StartupCommand",
        columns: vec!["Name", "Command", "Location"],
        filter: None,
    });
    m.insert("share", AliasEntry {
        class: "Win32_Share",
        columns: vec!["Name", "Path", "Description"],
        filter: None,
    });

    m
}

pub fn list_aliases() -> Vec<(&'static str, &'static str, String)> {
    let registry = build_alias_registry();
    let mut entries: Vec<_> = registry.iter()
        .map(|(name, entry)| {
            (*name, entry.class, entry.columns.join(", "))
        })
        .collect();
    entries.sort_by_key(|(name, _, _)| *name);
    entries
}
