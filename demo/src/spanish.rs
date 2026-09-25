//! The components' own words in Spanish: how an app adds a language (`i18n::languages` in
//! [`crate::router`]). The demo's own text stays English; switch with the language buttons
//! beside the theme toggle and every "Next", "Load more" and month name follows.

use loco_ui::i18n::{Strings, Text};

/// Every text of [`Text::ALL`], translated.
pub static SPANISH: Strings = Strings::new("es")
    .with(Text::Next, "Siguiente")
    .with(Text::Previous, "Anterior")
    .with(Text::Back, "Atrás")
    .with(Text::Skip, "Omitir")
    .with(Text::Finish, "Terminar")
    .with(Text::Close, "Cerrar")
    .with(Text::Cancel, "Cancelar")
    .with(Text::Dismiss, "Descartar")
    .with(Text::DismissMessage, "Descartar: {}")
    .with(Text::Submit, "Enviar")
    .with(Text::Search, "Buscar")
    .with(Text::TypeToSearch, "Escribe para buscar\u{2026}")
    .with(Text::NoMatches, "Sin resultados.")
    .with(Text::Create, "Crear")
    .with(Text::CreateValue, "Crear \u{201c}{}\u{201d}")
    .with(Text::RemoveValue, "Quitar {}")
    .with(Text::Selected, "Elegidos")
    .with(Text::Results, "Resultados")
    .with(Text::LoadMore, "Cargar más")
    .with(Text::ShowingOf, "Mostrando {} de {}")
    .with(Text::Loading, "Cargando")
    .with(Text::Pages, "Páginas")
    .with(Text::Page, "Página")
    .with(Text::OfTotal, "de {}")
    .with(Text::RangeOf, "{}\u{2013}{} de {}")
    .with(Text::Go, "Ir")
    .with(Text::RowsPerPage, "Filas por página")
    .with(Text::Show, "Mostrar")
    .with(Text::NoRows, "Ninguna fila coincide.")
    .with(Text::Columns, "Columnas")
    .with(Text::Edit, "Editar")
    .with(Text::Save, "Guardar")
    .with(Text::ExpandAll, "Abrir todo")
    .with(Text::CollapseAll, "Cerrar todo")
    .with(Text::Breadcrumb, "Ruta")
    .with(Text::ShowMore, "Mostrar {} más")
    .with(Text::PreviousMonth, "Mes anterior")
    .with(Text::NextMonth, "Mes siguiente")
    .with(Text::LongDate, "{0}, {1} de {2} de {3}")
    .with(Text::PickDate, "Elige una fecha")
    .with(Text::Opacity, "Opacidad {}")
    .with(Text::Presets, "Muestras")
    .with(Text::UseValue, "Usar {}")
    .with(Text::Reset, "Reiniciar")
    .with(Text::Value, "Valor")
    .with(Text::Set, "Fijar")
    .with(Text::FromTo, "de {} a {}")
    .with(Text::AtLeast, "al menos {}")
    .with(Text::AtMost, "como mucho {}")
    .with(Text::InStepsOf, ", de {0} en {0}")
    .with(Text::NoCards, "Sin tarjetas")
    .with(Text::OverLimit, ", por encima del límite")
    .with(Text::MoveTo, "Mover {} a {}")
    .with(
        Text::NothingByThatName,
        "Nada con ese nombre. Prueba una palabra o elige de la lista.",
    )
    .with(Text::TypeCommand, "Escribe una orden o una página")
    .with(Text::Minimum, "Mínimo")
    .with(Text::Maximum, "Máximo")
    .with(Text::FilterOptions, "Filtrar opciones")
    .with(Text::Filter, "Filtrar")
    .with(Text::Tab, "Pestaña")
    .with(
        Text::ChooseFiles,
        "Elige archivos o suéltalos sobre el botón",
    )
    .with(
        Text::ChooseFile,
        "Elige un archivo o suéltalo sobre el botón",
    )
    .with(Text::Upload, "Subir")
    .with(Text::UploadedFiles, "Archivos subidos")
    .with(Text::Resumed, "Seguimos donde lo dejaste, en el paso {}.")
    .with(Text::StartOver, "Empezar de nuevo")
    .with(Text::HasErrors, " (con errores)")
    .with(Text::Progress, "Progreso")
    .with(Text::StepsDone, "{} de {} pasos hechos")
    .with(Text::Step, "Paso {}")
    .with(Text::Optional, "(opcional)")
    .with(Text::Problem, "Hay un problema")
    .with(Text::Skipped, "(omitido)")
    .with(Text::EditValue, "Editar {}")
    .with(Text::StepOf, "Paso {} de {}")
    .with(Text::DayMonthYear, "{0} de {1} de {2}")
    .with(Text::MonthYear, "{0} de {1}")
    .with(Text::First, "Primera")
    .with(Text::Last, "Última")
    .with(Text::FilterRows, "Filtrar filas")
    .with(Text::FilterRowsHint, "Filtrar filas\u{2026}")
    .with(Text::Clear, "Borrar")
    .with(Text::DownloadCsv, "Descargar CSV")
    .with(Text::Select, "Elegir")
    .with(Text::SelectRow, "Elegir {}")
    .with(Text::Actions, "Acciones")
    .with(Text::RowActions, "Acciones de la fila")
    .with(Text::WithSelected, "Con las filas elegidas:")
    .with(Text::OneMatch, "1 resultado")
    .with(Text::Matches, "{} resultados")
    .with(Text::ForQuery, "{} para \u{201c}{}\u{201d}")
    .with(Text::IsSelected, " elegido")
    .with(Text::Decrement, "restar")
    .with(Text::Increment, "sumar")
    .with(Text::January, "enero")
    .with(Text::February, "febrero")
    .with(Text::March, "marzo")
    .with(Text::April, "abril")
    .with(Text::May, "mayo")
    .with(Text::June, "junio")
    .with(Text::July, "julio")
    .with(Text::August, "agosto")
    .with(Text::September, "septiembre")
    .with(Text::October, "octubre")
    .with(Text::November, "noviembre")
    .with(Text::December, "diciembre")
    .with(Text::Monday, "lunes")
    .with(Text::Tuesday, "martes")
    .with(Text::Wednesday, "miércoles")
    .with(Text::Thursday, "jueves")
    .with(Text::Friday, "viernes")
    .with(Text::Saturday, "sábado")
    .with(Text::Sunday, "domingo");

/// The languages the demo offers, English first (the default).
pub static LANGUAGES: [&Strings; 2] = [&Strings::ENGLISH, &SPANISH];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_text_is_translated() {
        let same: Vec<Text> = Text::ALL
            .into_iter()
            .filter(|t| SPANISH.get(*t) == Strings::ENGLISH.get(*t))
            .filter(|t| !matches!(t, Text::DayMonthYear | Text::MonthYear))
            .collect();
        assert!(same.is_empty(), "left in English: {same:?}");
    }
}
