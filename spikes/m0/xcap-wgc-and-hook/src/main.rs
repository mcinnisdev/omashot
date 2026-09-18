use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

static HOOK_HITS: AtomicUsize = AtomicUsize::new(0);

fn wgc_spike() {
    let monitors = xcap::Monitor::all().expect("monitors");
    let m = monitors.into_iter().next().expect("a monitor");
    let (rec, rx) = m.video_recorder().expect("recorder");
    rec.start().expect("start");
    let start = Instant::now();
    let mut times = Vec::new();
    let mut size = (0, 0);
    let mut bytes = 0usize;
    while start.elapsed() < Duration::from_secs(3) {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(f) => {
                times.push(Instant::now());
                size = (f.width, f.height);
                bytes = f.raw.len();
            }
            Err(_) => {}
        }
    }
    rec.stop().expect("stop");
    let n = times.len();
    let mut gaps: Vec<f64> = times.windows(2).map(|w| (w[1] - w[0]).as_secs_f64() * 1000.0).collect();
    gaps.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let fps = n as f64 / start.elapsed().as_secs_f64();
    let p50 = gaps.get(gaps.len() / 2).copied().unwrap_or(0.0);
    let p95 = gaps.get(gaps.len() * 95 / 100).copied().unwrap_or(0.0);
    let max = gaps.last().copied().unwrap_or(0.0);
    println!("WGC frames={n} fps={fps:.1} size={}x{} bytes/frame={bytes} gap_ms p50={p50:.1} p95={p95:.1} max={max:.1}", size.0, size.1);
}

unsafe extern "system" fn hook_proc(code: i32, wparam: usize, lparam: isize) -> isize {
    use windows_sys::Win32::UI::WindowsAndMessaging::{CallNextHookEx, KBDLLHOOKSTRUCT, WM_KEYDOWN};
    if code >= 0 && wparam as u32 == WM_KEYDOWN {
        let k = &*(lparam as *const KBDLLHOOKSTRUCT);
        HOOK_HITS.fetch_add(1, Ordering::SeqCst);
        println!("  hook saw vk={} injected={}", k.vkCode, (k.flags & 0x10) != 0);
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

fn hook_spike() {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{SendInput, INPUT, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_F24};
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    let (tx, rx) = std::sync::mpsc::channel::<u32>();
    let t = std::thread::spawn(move || unsafe {
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), std::ptr::null_mut(), 0);
        if hook.is_null() {
            println!("HOOK install failed");
            let _ = tx.send(0);
            return;
        }
        let _ = tx.send(windows_sys::Win32::System::Threading::GetCurrentThreadId());
        let mut msg: MSG = std::mem::zeroed();
        while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
        UnhookWindowsHookEx(hook);
    });
    let tid = rx.recv().unwrap();
    if tid == 0 { return; }
    std::thread::sleep(Duration::from_millis(200));
    // F24 exists on no keyboard, so injecting it disturbs nothing.
    for _ in 0..3 {
        unsafe {
            let mut down: INPUT = std::mem::zeroed();
            down.r#type = INPUT_KEYBOARD;
            down.Anonymous.ki = KEYBDINPUT { wVk: VK_F24, wScan: 0, dwFlags: 0, time: 0, dwExtraInfo: 0 };
            let mut up = down;
            up.Anonymous.ki.dwFlags = KEYEVENTF_KEYUP;
            SendInput(1, &down, std::mem::size_of::<INPUT>() as i32);
            SendInput(1, &up, std::mem::size_of::<INPUT>() as i32);
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    std::thread::sleep(Duration::from_millis(200));
    unsafe { PostThreadMessageW(tid, WM_QUIT, 0, 0); }
    let _ = t.join();
    println!("HOOK keydown events seen={} (expected 3)", HOOK_HITS.load(Ordering::SeqCst));
}

fn main() {
    wgc_spike();
    hook_spike();
}
