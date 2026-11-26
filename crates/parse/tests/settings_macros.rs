use jjpwrgem_parse::settings::{Table, choice, choice_enum, integer, settings};

choice_enum! {
    pub enum Mode default Fast {
        /// Quick.
        Fast = "fast",
        /// Careful.
        Slow = "slow",
    }
}

settings! {
    struct Demo, table DEMO_TABLE, id DemoId {
        /// How many.
        Count count: Option<usize> = integer(3),
        /// Which mode.
        Mode mode: Mode = choice().shorthand("slow", Some('s'), "slow"),
    }
}

#[test]
fn external_crate_defines_settings() {
    let demo = Demo {
        count: Some(7),
        mode: Mode::Slow,
    };
    assert_eq!(demo.value_text(DemoId::Count), "7");
    assert_eq!(demo.value_text(DemoId::Mode), "slow");
    assert_eq!(<Demo as Table>::TABLE.len(), 2);
    assert_eq!(DemoId::Count.setting().name, "count");
}
