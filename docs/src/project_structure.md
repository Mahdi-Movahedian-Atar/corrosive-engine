# Project Structure

Projects that use this engine must be structured as the following:
- src
    - .corrosive-engine
    - comp
        - mod.rs
        - OTHER_FOLDERS
    - task
        - mod.rs
        - OTHER_FOLDERS
    - main.rs
- build.rs
---
- The `.corrosive_engine` contains files that are generated by the engine.
- The comp folder is where components, resources, states, and traits are located.
- The task files are where tasks are located.
- The main.rs is where the `corrosive-engine-builder` macro is located.
- The build.rs creates the engine at build time.