use color_print::{cprintln, cstr};

use crate::Config;

pub const BANNER: &str = color_print::cstr! {
r#"
<#FFFFFF>████████╗</#FFFFFF><#999999>██╗  ██╗</#999999><#FF007F>██████╗ </#FF007F>
<#FFFFFF>╚══██╔══╝</#FFFFFF><#999999>╚██╗██╔╝</#999999><#FF007F>╚════██╗</#FF007F>
<#FFFFFF>   ██║   </#FFFFFF><#999999> ╚███╔╝ </#999999><#FF007F> █████╔╝</#FF007F>
<#FFFFFF>   ██║   </#FFFFFF><#999999> ██╔██╗ </#999999><#FF007F> ╚═══██╗</#FF007F>
<#FFFFFF>   ██║   </#FFFFFF><#999999>██╔╝ ██╗</#999999><#FF007F>██████╔╝</#FF007F>
<#FFFFFF>   ╚═╝   </#FFFFFF><#999999>╚═╝  ╚═╝</#999999><#FF007F>╚═════╝ </#FF007F>"#
};

pub fn print_banner(config: &Config) {
    println!("\n{}\n", BANNER.trim_start());

    cprintln!(
        "root dir: <#FFFFFF>{}</#FFFFFF>",
        config.root_dir().display()
    );
    cprintln!("channel: <#FFFFFF>{}</#FFFFFF>", config.ensure_channel());
    println!();
}
