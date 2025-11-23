use std::panic;

use color_eyre::eyre::Result;

pub fn init() -> Result<()> {
    use color_eyre::config::HookBuilder;
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();
    eyre_hook.install()?;

    panic::set_hook(Box::new(move |panic_info| {
        if let Ok(mut tui) = crate::tui::Tui::new()
            && let Err(e) = tui.exit()
        {
            tracing::error!("Unable to exit Terminal: {:?}", e);
        }

        tracing::error!("Panic: {}", panic_info);
        eprintln!("{}", panic_hook.panic_report(panic_info));

        std::process::exit(1);
    }));
    Ok(())
}
