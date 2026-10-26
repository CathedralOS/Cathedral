//! Drive Omega's realm setup with host-owned images, grants and exact task records.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Job {
    Configuration = 1,
    Picker,
    Background,
    Discovery,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u64)]
pub enum Command {
    Refresh = 1,
    ReadConfiguration,
    ValidateConfiguration,
    AdmitLauncher,
    ReleaseImage,
    BindLauncher,
    InstallWorkspace,
    PlaceLauncher,
    StartBackground,
    BackgroundFailed,
    Flush,
    Notify,
    GrantChrome,
    RecordLauncher,
    StopLauncher,
    ClearWorkspace,
    ResetSurfaces,
    PreparePicker,
    ReadPicker,
    ValidatePicker,
    AdmitPicker,
    BindPicker,
    PlacePicker,
    InstallPicker,
    RecordPicker,
    ReadBackground,
    AdmitBackground,
    PlaceBackground,
    RecordBackground,
    StopBackground,
    InspectRealm,
    ConfigureRealm,
    StartPicker,
    MarkHosted,
    RecoverStartup,
    Finish,
}
pub struct State {
    words: [u64; 5],
}
impl State {
    pub fn new(job: Job, configured: bool, busy: bool, discover: bool) -> Self {
        let mut state = Self { words: [0; 5] };
        assert_eq!(
            state.apply(
                0,
                [job as u64, configured.into(), busy.into(), discover.into()]
            ),
            1
        );
        state
    }
    pub fn command(&mut self) -> Option<Command> {
        use Command::*;
        Some(match self.apply(1, [0; 4]) {
            0 => return None,
            1 => Refresh,
            2 => ReadConfiguration,
            3 => ValidateConfiguration,
            4 => AdmitLauncher,
            5 => ReleaseImage,
            6 => BindLauncher,
            7 => InstallWorkspace,
            8 => PlaceLauncher,
            9 => StartBackground,
            10 => BackgroundFailed,
            11 => Flush,
            12 => Notify,
            13 => GrantChrome,
            14 => RecordLauncher,
            15 => StopLauncher,
            16 => ClearWorkspace,
            17 => ResetSurfaces,
            18 => PreparePicker,
            19 => ReadPicker,
            20 => ValidatePicker,
            21 => AdmitPicker,
            22 => BindPicker,
            23 => PlacePicker,
            24 => InstallPicker,
            25 => RecordPicker,
            26 => ReadBackground,
            27 => AdmitBackground,
            28 => PlaceBackground,
            29 => RecordBackground,
            30 => StopBackground,
            31 => InspectRealm,
            32 => ConfigureRealm,
            33 => StartPicker,
            34 => MarkHosted,
            35 => RecoverStartup,
            36 => Finish,
            _ => unreachable!("Omega realm operation"),
        })
    }
    pub fn complete(&mut self, command: Command, status: u64, detail: u64) -> bool {
        self.apply(2, [command as u64, status, detail, 0]) == 1
    }
    pub fn status(&self) -> u64 {
        self.words[2]
    }
    pub fn last_status(&self) -> u64 {
        self.words[3]
    }
    #[inline(never)]
    fn apply(&mut self, event: u64, facts: [u64; 4]) -> u64 {
        let mut p = [0; 128];
        p[0] = 16;
        p[1] = event;
        p[2..6].copy_from_slice(&facts);
        p[16..21].copy_from_slice(&self.words);
        let result = super::native::decide(&mut p);
        self.words.copy_from_slice(&p[16..21]);
        result
    }
}
