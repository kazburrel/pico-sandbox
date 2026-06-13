#![no_std]
#![no_main]

// We need to write our own panic handler
use core::panic::PanicInfo;

// Import traits for embedded abstractions
use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;


// Alias our HAL
use rp235x_hal as hal;

// Custom panic handler: just loop forever
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

// Copy boot metadata to .start_block so boot ROM knows how to boot our program
#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: hal::block::ImageDef = hal::block::ImageDef::secure_exe();

// Set external crystal frequency
const XOSC_CRYSTAL_FREQ: u32 = 12_000_000;

// Our possible LED modes.
// Clone + Copy lets us reuse current_mode inside the loop without moving it.
#[derive(Clone, Copy)]
#[allow(dead_code)]
enum BlinkMode {
    Normal,
    Heartbeat,
    Panic,
    Sos,
}

// Main entrypoint (custom defined for embedded targets)
#[hal::entry]
fn main() -> ! {
    // Get ownership of hardware peripherals
    let mut pac = hal::pac::Peripherals::take().unwrap();

    // Set up the watchdog and clocks
    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    // Start the system clocks and PLLs.
    let clocks = hal::clocks::init_clocks_and_plls(
        XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    // Single-cycle I/O block (fast GPIO)
    let sio = hal::Sio::new(pac.SIO);


    // Split off ownership of Peripherals struct, set pins to default state
    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // Configure pin, get ownership of that pin
    let mut led_pin = pins.gpio25.into_push_pull_output();

    // Move ownership of TIMER0 peripheral to create Timer struct
    let mut timer = hal::Timer::new_timer0(
        pac.TIMER0,
        &mut pac.RESETS,
        &clocks,
    );

    // let mut delay = hal::Timer::new_timer0(pac.TIMER0, &mut pac.RESETS);
    let short_delay: u32 = 100;
    let long_delay: u32 = 700;

    // Pick the current LED mode.
    // Change this to Normal, Heartbeat, or Panic.
   let current_mode: BlinkMode = BlinkMode::Sos;
    // Main program loop.
    loop {
        // Pick the blink pattern based on current_mode.
        match current_mode {
            BlinkMode::Normal => {
                // Normal blink: ON 500ms, OFF 500ms.
                blink_once(&mut led_pin, &mut timer, 500, 500);
            }

            BlinkMode::Heartbeat => {
                // First quick blink: ON 100ms, OFF 100ms.
                blink_once(&mut led_pin, &mut timer, short_delay, short_delay);

                // Second quick blink: ON 100ms, then long OFF pause.
                blink_once(&mut led_pin, &mut timer, short_delay, long_delay);
            }

            BlinkMode::Panic => {
                // Panic blink: very fast blinking.
                blink_once(&mut led_pin, &mut timer, 50, 50);
            }

            BlinkMode::Sos => {
                // 3 short (S)
                blink_once(&mut led_pin, &mut timer, short_delay, short_delay);
                blink_once(&mut led_pin, &mut timer, short_delay, short_delay);
                blink_once(&mut led_pin, &mut timer, short_delay, short_delay);

                // gap between letters
                timer.delay_ms(long_delay);

                // 3 long (O)
                blink_once(&mut led_pin, &mut timer, long_delay, short_delay);
                blink_once(&mut led_pin, &mut timer, long_delay, short_delay);
                blink_once(&mut led_pin, &mut timer, long_delay, short_delay);

                // gap between letters
                timer.delay_ms(long_delay);

                // 3 short (S)
                blink_once(&mut led_pin, &mut timer, short_delay, short_delay);
                blink_once(&mut led_pin, &mut timer, short_delay, short_delay);
                blink_once(&mut led_pin, &mut timer, short_delay, short_delay);

                // long pause before repeating
                timer.delay_ms(2000);
            }
        }
    }
}

// Blink the LED once using the given ON and OFF delay times.
//
// led_pin and timer are borrowed mutably because:
// - led_pin changes state when we call set_high() / set_low()
// - timer is used to wait
fn blink_once<PIN, TIMER>(
    led_pin: &mut PIN,
    timer: &mut TIMER,
    on_delay: u32,
    off_delay: u32,
)
where
    // PIN can be any type that behaves like an output pin.
    PIN: OutputPin,

    // TIMER can be any type that behaves like a delay timer.
    TIMER: DelayNs,
{
    // Turn LED on.
    led_pin.set_high().unwrap();

    // Keep it on for on_delay milliseconds.
    timer.delay_ms(on_delay);

    // Turn LED off.
    led_pin.set_low().unwrap();

    // Keep it off for off_delay milliseconds.
    timer.delay_ms(off_delay);
}