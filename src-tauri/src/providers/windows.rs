use crate::model::Monitor;
use std::ffi::c_void;
type Handle = *mut c_void;
#[repr(C)]
struct Rect {
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}
#[repr(C)]
struct Info {
    size: u32,
    monitor: Rect,
    work: Rect,
    flags: u32,
    device: [u16; 32],
}
#[repr(C)]
struct Physical {
    handle: Handle,
    description: [u16; 128],
}
#[link(name = "user32")]
unsafe extern "system" {
    fn EnumDisplayMonitors(
        dc: Handle,
        clip: *const Rect,
        callback: unsafe extern "system" fn(Handle, Handle, *mut Rect, isize) -> i32,
        data: isize,
    ) -> i32;
    fn GetMonitorInfoW(monitor: Handle, info: *mut Info) -> i32;
}
#[link(name = "dxva2")]
unsafe extern "system" {
    fn GetNumberOfPhysicalMonitorsFromHMONITOR(monitor: Handle, count: *mut u32) -> i32;
    fn GetPhysicalMonitorsFromHMONITOR(monitor: Handle, count: u32, monitors: *mut Physical)
        -> i32;
    fn DestroyPhysicalMonitors(count: u32, monitors: *const Physical) -> i32;
    fn SetVCPFeature(monitor: Handle, code: u8, value: u32) -> i32;
    fn GetVCPFeatureAndVCPFeatureReply(
        monitor: Handle,
        code: u8,
        kind: *mut u32,
        current: *mut u32,
        maximum: *mut u32,
    ) -> i32;
}
struct Entry {
    id: String,
    name: String,
    physical: Physical,
}
struct Handles(Vec<Entry>);
impl Drop for Handles {
    fn drop(&mut self) {
        for entry in &self.0 {
            unsafe {
                DestroyPhysicalMonitors(1, &entry.physical);
            }
        }
    }
}
fn wide(value: &[u16]) -> String {
    String::from_utf16_lossy(&value[..value.iter().position(|c| *c == 0).unwrap_or(value.len())])
}
unsafe extern "system" fn callback(handle: Handle, _: Handle, _: *mut Rect, data: isize) -> i32 {
    let entries = &mut *(data as *mut Vec<Entry>);
    let mut info: Info = std::mem::zeroed();
    info.size = std::mem::size_of::<Info>() as u32;
    if GetMonitorInfoW(handle, &mut info) == 0 {
        return 1;
    }
    let mut count = 0;
    if GetNumberOfPhysicalMonitorsFromHMONITOR(handle, &mut count) == 0 || count == 0 || count > 32
    {
        return 1;
    }
    let mut monitors: Vec<Physical> = (0..count).map(|_| std::mem::zeroed()).collect();
    if GetPhysicalMonitorsFromHMONITOR(handle, count, monitors.as_mut_ptr()) == 0 {
        return 1;
    }
    for (index, physical) in monitors.into_iter().enumerate() {
        entries.push(Entry {
            id: format!("ddc:{}:{index}", wide(&info.device)),
            name: wide(&physical.description),
            physical,
        });
    }
    1
}
fn handles() -> Result<Handles, String> {
    let mut result = Handles(vec![]);
    if unsafe {
        EnumDisplayMonitors(
            std::ptr::null_mut(),
            std::ptr::null(),
            callback,
            &mut result.0 as *mut _ as isize,
        )
    } == 0
    {
        return Err(std::io::Error::last_os_error().to_string());
    }
    Ok(result)
}
pub fn enumerate() -> Result<Vec<Monitor>, String> {
    Ok(handles()?
        .0
        .iter()
        .map(|e| Monitor {
            id: e.id.clone(),
            name: e.name.clone(),
            provider: "Windows DDC".into(),
            detail: "Standard MCCS input selection • verify assignments after cabling changes"
                .into(),
            input_presets: vec![],
        })
        .collect())
}
pub fn set_input(id: &str, input: u16) -> Result<(), String> {
    let handles = handles()?;
    let entry = handles
        .0
        .iter()
        .find(|e| e.id == id)
        .ok_or("Monitor is unavailable; refresh inventory")?;
    if unsafe { SetVCPFeature(entry.physical.handle, 0x60, u32::from(input)) } == 0 {
        return Err(format!(
            "DDC input request failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}
pub fn get_input(id: &str) -> Result<u16, String> {
    let handles = handles()?;
    let entry = handles
        .0
        .iter()
        .find(|e| e.id == id)
        .ok_or("Monitor is unavailable; refresh inventory")?;
    let (mut kind, mut current, mut maximum) = (0, 0, 0);
    if unsafe {
        GetVCPFeatureAndVCPFeatureReply(
            entry.physical.handle,
            0x60,
            &mut kind,
            &mut current,
            &mut maximum,
        )
    } == 0
    {
        return Err(format!(
            "DDC input read failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    u16::try_from(current).map_err(|_| "Monitor returned an invalid input value".into())
}
