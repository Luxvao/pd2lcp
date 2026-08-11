# PD2LCP
PD2LCP is a modern cross-platform ish (more on that [here](#platforms)) Project Diablo 2 launcher written with Rust and Iced. It exists to enhance the user experience on certain unsupported platforms.

## Platforms
PD2LCP is designed primarily for Linux and Android (running under Gamenative has been tested, Winlator/Gamehub should work fine too). On Linux it's a native app that installs and manages its own Wine prefix and directories, no Bottles needed. On Android it's
designed to run under Gamenative as a Windows executable (that does not mean it's compatible with Windows itself, PD2LCP makes some assumptions about the environment that do not hold on Windows, namely that A:\\ exists).

## Installation
### Both
It is **VERY IMPORTANT** to **ALWAYS** install the base D2 game under "A:\\Diablo II". PD2LCP assumes you're installing there and will not work otherwise. This is to make sure it functions well under Gamenative.

The installers themselves can prove quite janky. One specific quirk of theirs is that I cannot accurately track when the installation of the base game completed successfully, therefore it's important to let the installers finish. After the first downloader
exits, there will also be a second proper installer, and only after you've gone through that one is the game installed. If anything's broken by the end, there is a reset button in the settings that will **WIPE THE ENTIRE INSTALL**, letting you start from scratch.

### Linux
Installation on Linux is fairly simple.
#### Prerequisites
  - Official D2 and D2 LOD installers from battle.net
  - Valid CD keys

#### Process
  0. Make sure to read the [Both](#both) section.
  1. Download `pd2lcp-iced` (or flatpak; COMING SOON).
  2. Open it, you will see a welcome screen. Just press Get Started.
  3. Now you're on the "Select D2 Installer" screen. Quite self explanatory, just click on the button and select the D2 installer.
  4. Same as step 3. just for the LOD installer.
  5. Now you're on the main screen. Press "Launch" and enjoy!


### Android
Android is a bit more involved, since you're in an externally managed Wine environment.
#### Prerequisites
  - Official D2 and D2 LOD installers from battle.net
  - Valid CD keys
  - Gamenative (preferred, tested) OR Winlator OR Gamehub

#### Process
  0. Make sure to read the [Both](#both) section.
  1. Download `pd2lcp-iced.exe`.
  2. Go into your Files app and create a new folder anywhere (I personally like to call it "Project Diablo 2", which will be mirrored in Gamenative). This will be the base for your Gamenative container.
  3. Open Gamenative and add a custom game (little plus icon in the top right). It is very important to be on the **Github** version of Gamenative, as the Play Store one does not contain that button or the ability to add custom games.
  4. Move `pd2lcp-iced.exe` into the folder you made in step #2 (don't rename the executable, it's important).
  5. Open container settings in Gamenative and set the executable to `pd2lcp-iced.exe`. You must also set the Proton version to the `x86_64` one, I haven't had the installers open under arm ec builds.
  6. Now just launch the game and the launcher will open.
  7. You will be prompted to select the D2 installer. The D:\\ drive will point to your phone's Downloads folder, so the installers should be there. I haven't figured out how to paste the CD keys under Gamenative so you'll have to write them manually.
  8. Do the same for LOD.
  9. Press "Launch" and enjoy!

## Runtime Flags
- `-gamemode` - Launches PD2LCP in full screen mode (does nothing on Android; gamemode is the default there)
- `-skiplauncher` - Skips loading the launcher UI and just starts the game (it will still check for updates and if it finds any, it will open the UI to show progress, then launch)
