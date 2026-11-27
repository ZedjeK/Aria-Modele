//! Les boîtes de dialogue de Windows, pour ce qui doit être dit hors de toute fenêtre de l'appli : l'appli ne peut
//! pas démarrer (WebView2 absent, par exemple), ou un avertissement au moment de se relancer.

use crate::identite;

/// Affiche pourquoi l'appli n'a pas pu démarrer.
#[cfg(windows)]
pub fn afficher(message: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MB_OK, MessageBoxW};
    use windows::core::HSTRING;

    let titre = HSTRING::from(identite::nom());
    let texte = HSTRING::from(format!("{} n'a pas pu démarrer.\n\n{message}", identite::nom()));
    // SAFETY : les deux textes sont des HSTRING valides jusqu'à la fin de l'appel ; aucune fenêtre parente.
    #[allow(unsafe_code)]
    unsafe {
        MessageBoxW(None, &texte, &titre, MB_OK | MB_ICONERROR);
    }
}

/// Hors de Windows (tests, outils) : sur la sortie d'erreur.
#[cfg(not(windows))]
pub fn afficher(message: &str) {
    eprintln!("{} n'a pas pu démarrer : {message}", identite::nom());
}

/// Prévient d'un problème qui n'empêche pas de continuer (un raccourci non créé, par exemple).
#[cfg(windows)]
pub fn avertir(message: &str) {
    use windows::Win32::UI::WindowsAndMessaging::{MB_ICONWARNING, MB_OK, MessageBoxW};
    use windows::core::HSTRING;

    let titre = HSTRING::from(identite::nom());
    let texte = HSTRING::from(message);
    // SAFETY : comme pour `afficher`.
    #[allow(unsafe_code)]
    unsafe {
        MessageBoxW(None, &texte, &titre, MB_OK | MB_ICONWARNING);
    }
}

/// Hors de Windows : sur la sortie d'erreur.
#[cfg(not(windows))]
pub fn avertir(message: &str) {
    eprintln!("{} : {message}", identite::nom());
}

/// La page de Microsoft qui installe WebView2 (« Microsoft Edge WebView2 Runtime ») sur le PC d'un utilisateur.
pub const PAGE_WEBVIEW2: &str = "https://developer.microsoft.com/microsoft-edge/webview2/consumer/";

/// Ce que dit l'appli quand WebView2 manque : ce que c'est, quoi faire, et la question d'ouvrir la page.
fn texte_webview2_absent() -> String {
    format!(
        "{} ne peut pas s'ouvrir : WebView2 n'est pas installé sur ce PC.\n\n\
         WebView2 est le moteur d'affichage de Windows qui dessine la fenêtre de l'appli. Il est présent \
         d'office sur Windows 11 et sur un Windows 10 22H2 à jour.\n\n\
         Installe « Microsoft Edge WebView2 Runtime » depuis la page de Microsoft, puis relance l'appli :\n\
         {PAGE_WEBVIEW2}\n\n\
         Ouvrir cette page maintenant ?",
        identite::nom()
    )
}

/// WebView2 manque : le dit, et propose d'ouvrir la page de Microsoft pour l'installer.
#[cfg(windows)]
pub fn webview2_absent() {
    use std::process::Command;

    use windows::Win32::UI::WindowsAndMessaging::{IDYES, MB_ICONERROR, MB_YESNO, MessageBoxW};
    use windows::core::HSTRING;

    let titre = HSTRING::from(identite::nom());
    let texte = HSTRING::from(texte_webview2_absent());
    // SAFETY : les deux textes sont des HSTRING valides jusqu'à la fin de l'appel ; aucune fenêtre parente.
    #[allow(unsafe_code)]
    let reponse = unsafe { MessageBoxW(None, &texte, &titre, MB_YESNO | MB_ICONERROR) };
    if reponse == IDYES {
        // Le navigateur par défaut ; s'il ne s'ouvre pas, l'adresse est écrite dans le message.
        let _ = Command::new("explorer").arg(PAGE_WEBVIEW2).spawn();
    }
}

/// Hors de Windows : sur la sortie d'erreur.
#[cfg(not(windows))]
pub fn webview2_absent() {
    eprintln!("{}", texte_webview2_absent());
}
