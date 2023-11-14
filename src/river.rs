use tokio::process::Command;
use tracing::trace;
#[tracing::instrument]
pub async fn run(
    border_width: i8,
    colors: &Colors,
    hardware: &Hardware,
    startup_commands: &Vec<CommandSet>,
    apps: &Apps,
) -> Result<(), Error> {
    trace!("starting");
    trace!("async: starting tags");
    let border_width_as_string = border_width.to_string();
    let player_pause = format!("{} play-pause", apps.player_ctl);

    let player_previous = format!("{} previous", apps.player_ctl);
    let player_next = format!("{} next", apps.player_ctl);
    let handlers = [
        vec!["background-color", &colors.background],
        vec!["border-color-focused", &colors.border_focused],
        vec!["border-color-unfocused", &colors.border_unfocused],
        vec!["border-color-urgent", &colors.border_urgent],
        vec!["border-width", &border_width_as_string],
        vec!["input", &hardware.pointer.name, "drag", "enabled"],
        vec!["input", &hardware.pointer.name, "tap", "enabled"],
        vec!["input", &hardware.pointer.name, "events", "enabled"],
        vec!["input", &hardware.pointer.name, "natural-scroll", "enabled"],
        vec![
            "input",
            &hardware.pointer.name,
            "scroll-method",
            "two-finger",
        ],
        vec!["input", &hardware.lid.name, "events", "enable"],
        vec![
            "map-switch",
            "normal",
            "lid",
            "open",
            "spawn",
            &hardware.lid.on_lid_open,
        ],
        vec![
            "map-switch",
            "normal",
            "lid",
            "close",
            "spawn",
            &hardware.lid.on_lid_close,
        ],
        vec!["set-repeat", "50", "300"],
        vec!["float-filter-add", "app-id", "Rofi"],
        vec!["float-filter-add", "app-id", "Fuzzel"],
        vec!["float-filter-add", "app-id", "float"],
        vec!["float-filter-add", "app-id", "popup"],
        vec!["float-filter-add", "app-id", "pinentry-qt"],
        vec!["float-filter-add", "app-id", "pinentry-gtk"],
        vec!["float-filter-add", "title", "Picture-in-Picture"],
        vec!["float-filter-add", "app-id", "launcher"],
        vec!["csd-filter-add", "app-id", "Rofi"],
        vec!["csd-filter-add", "app-id", "Fuzzel"],
        vec!["csd-filter-add", "app-id", "launcher"],
        vec!["focus-follows-cursor", "normal"],
        vec!["set-cursor-warp", "no-output-change"],
        vec!["attach-mode", "bottom"],
        vec!["default-layout", "rivertile"],
        vec!["map-pointer", "normal", "Super", "BTN_LEFT", "move-view"],
        vec!["map", "normal", "Super", "BTN_RIGHT", "resize-view"],
        vec!["map", "normal", "Super", "Return", "spawn", &apps.terminal],
        vec!["map", "normal", "Super", "D", "spawn", &apps.launcher],
        vec!["map", "normal", "Super", "J", "focus-view", "next"],
        vec!["map", "normal", "Super", "K", "focus-view", "previous"],
        vec!["map", "normal", "Super", "space", "zoom"],
        vec!["map", "normal", "Super", "Q", "close"],
        vec!["map", "normal", "Super", "Period", "focus-output", "next"],
        vec![
            "map",
            "normal",
            "Super",
            "Comma",
            "focus-output",
            "previous",
        ],
        vec![
            "map",
            "normal",
            "Super+Shift",
            "Period",
            "send-to-output",
            "next",
        ],
        vec![
            "map",
            "normal",
            "Super+Shift",
            "Comma",
            "send-to-output",
            "previous",
        ],
        vec![
            "map",
            "normal",
            "Super",
            "H",
            "send-layout-cmd",
            "rivertile",
            "main-ratio -0.05",
        ],
        vec![
            "map",
            "normal",
            "Super",
            "L",
            "send-layout-cmd",
            "rivertile",
            "main-ratio +0.05",
        ],
        vec![
            "map",
            "normal",
            "Super+Alt+Shift",
            "H",
            "resize",
            "horizontal -100",
        ],
        vec![
            "map",
            "normal",
            "Super+Alt+Shift",
            "J",
            "resize",
            "vertical 100",
        ],
        vec![
            "map",
            "normal",
            "Super+Alt+Shift",
            "K",
            "resize",
            "vertical -100",
        ],
        vec![
            "map",
            "normal",
            "Super+Alt+Shift",
            "L",
            "resize",
            "horizontal 100",
        ],
        vec!["map", "normal", "Super+Shift", "F", "toggle-float"],
        vec!["map", "normal", "Super", "F", "toggle-fullscreen"],
        vec![
            "map",
            "normal",
            "Super",
            "Up",
            "send-layout-cmd",
            "rivertile",
            "main-location top",
        ],
        vec![
            "map",
            "normal",
            "Super",
            "Right",
            "send-layout-cmd",
            "rivertile",
            "main-location right",
        ],
        vec![
            "map",
            "normal",
            "Super",
            "Down",
            "send-layout-cmd",
            "rivertile",
            "main-location bottom",
        ],
        vec![
            "map",
            "normal",
            "Super",
            "Left",
            "send-layout-cmd",
            "rivertile",
            "main-location left",
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86AudioMedia",
            "spawn",
            &player_pause,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86AudioPlay",
            "spawn",
            &player_pause,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86AudioPrev",
            "spawn",
            &player_previous,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86AudioNext",
            "spawn",
            &player_next,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86AudioRaiseVolume",
            "spawn",
            &apps.volume_up,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86AudioLowerVolume",
            "spawn",
            &apps.volume_down,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86AudioMute",
            "spawn",
            &apps.volume_mute,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86MonBrightnessUp",
            "spawn",
            &apps.brightness_up,
        ],
        vec![
            "map",
            "normal",
            "None",
            "XF86MonBrightnessDown",
            "spawn",
            &apps.brightness_down,
        ],
    ]
    .into_iter()
    .map(riverctl);
    let start_up_handlers = startup_commands.iter().map(command_runner);
    tags().await?;
    trace!("defined the tags");
    for handle in handlers {
        handle.await?;
    }
    trace!("done with the general handlers");
    for handle in start_up_handlers {
        handle.await?;
    }
    trace!("done with the startup handlers");

    Ok(())
}
#[tracing::instrument]
async fn command_runner(cmd: &CommandSet) -> Result<(), Error> {
    trace!("started runnig a command");
    Command::new(&cmd.executible)
        .args(&cmd.args)
        .spawn()
        .map_err(Error::CouldNotSpawnTheTask)?
        .wait_with_output()
        .await
        .map_err(Error::TaskReturnedError)?;
    trace!("done running a command");
    Ok(())
}

async fn riverctl(args: Vec<&str>) -> Result<(), Error> {
    command_runner(&CommandSet {
        executible: "riverctl".to_owned(),
        args: args.into_iter().map(ToOwned::to_owned).collect(),
    })
    .await?;
    Ok(())
}

#[tracing::instrument]
async fn tags() -> Result<(), Error> {
    static SET_FOCUS: &str = "set-focused-tags";
    static TOGGLE_FOCUS: &str = "toggle-focused-tags";
    static TOGGLE_VIEW: &str = "toggle-view-tags";
    static SET_VIEW: &str = "set-view-tags";

    for i in 1..=9 {
        let numb = format!("{}", i);
        let tag = format!("{}", 1 << (i - 1));

        let que: Vec<Vec<&str>> = vec![
            vec!["map", "normal", "Super", &numb, SET_FOCUS, &tag],
            vec!["map", "normal", "Super+Shift", &numb, SET_VIEW, &tag],
            vec!["map", "normal", "Super+Control", &numb, TOGGLE_FOCUS, &tag],
            vec![
                "map",
                "normal",
                "Super+Shift+Control",
                &numb,
                TOGGLE_VIEW,
                &tag,
            ],
        ];

        let mut handles = vec![];
        for command in que.iter() {
            handles.push(riverctl(command.to_vec()));
        }
        for handle in handles {
            handle.await?;
        }
    }

    let all_tags = format!("{}", (1u64 << 32) - 1);
    riverctl(vec!["map", "normal", "Super", "0", SET_FOCUS, &all_tags]).await?;
    riverctl(vec![
        "map",
        "normal",
        "Super+Shift",
        "0",
        SET_VIEW,
        &all_tags,
    ])
    .await?;
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("could not spawn the task: {0}")]
    CouldNotSpawnTheTask(std::io::Error),
    #[error("could not spawn the task: {0}")]
    TaskReturnedError(std::io::Error),
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Colors {
    pub background: String,
    pub border_focused: String,
    pub border_unfocused: String,
    pub border_urgent: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Hardware {
    pub pointer: Pointer,
    pub lid: Lid,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Pointer {
    pub name: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Lid {
    pub name: String,
    pub on_lid_close: String,
    pub on_lid_open: String,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Apps {
    pub terminal: String,
    pub launcher: String,
    pub player_ctl: String,
    pub volume_down: String,
    pub volume_up: String,
    pub volume_mute: String,
    pub brightness_up: String,
    pub brightness_down: String,
}
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CommandSet {
    executible: String,
    args: Vec<String>,
}
