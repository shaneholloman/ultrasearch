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
        CopySelectedFile,
        QuitApp,
        FinishOnboarding,
        OpenContainingFolder,
        ShowProperties
    ]
);
