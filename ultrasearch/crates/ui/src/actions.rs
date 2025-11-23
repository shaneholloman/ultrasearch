use gpui::actions;

actions!(
    ultrasearch,
    [
        FocusSearch,
        ClearSearch,
        SubmitSearch,
        SelectNext,
        SelectPrev,
        OpenSelected,
        ModeMetadata,
        ModeMixed,
        ModeContent,
        CopySelectedPath,
        QuitApp,
        FinishOnboarding,
        OpenContainingFolder,
        ShowProperties
    ]
);
