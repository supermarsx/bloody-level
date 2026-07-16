use rusqlite::{params, Connection};

use super::{
    canonical::AnalyteRegistry, header, preprocess, rows::parse_row, sections::SectionContext,
    ParsedRow, RangeGrammar,
};

fn registry() -> AnalyteRegistry {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch(
        "CREATE TABLE analytes(id TEXT PRIMARY KEY, pt_name TEXT NOT NULL);
         CREATE TABLE analyte_aliases(alias TEXT PRIMARY KEY, analyte_id TEXT NOT NULL, source TEXT NOT NULL);",
    )
    .unwrap();

    for (id, name) in [
        ("hemoglobina", "Hemoglobina"),
        ("eritrocitos", "Eritrócitos"),
        ("neutrofilos", "Neutrófilos"),
        ("colesterol_ldl", "Colesterol LDL"),
        ("ferritina", "Ferritina"),
        ("tsh", "Tireoestimulina (TSH)"),
        ("vitamina_d_25oh", "Vitamina D (25-OH)"),
    ] {
        conn.execute(
            "INSERT INTO analytes(id, pt_name) VALUES(?1, ?2)",
            params![id, name],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO analyte_aliases(alias, analyte_id, source) VALUES(?1, ?2, 'test')",
            params![name, id],
        )
        .unwrap();
    }

    for (alias, id) in [
        ("TSH", "tsh"),
        ("Vitamina D", "vitamina_d_25oh"),
        ("25-OH Vitamina D", "vitamina_d_25oh"),
    ] {
        conn.execute(
            "INSERT INTO analyte_aliases(alias, analyte_id, source) VALUES(?1, ?2, 'test')",
            params![alias, id],
        )
        .unwrap();
    }

    AnalyteRegistry::load_from_db(&conn).unwrap()
}

fn parse_report_text(text: &str) -> Vec<ParsedRow> {
    let reg = registry();
    let normalized = preprocess::normalize(text);
    let mut ctx = SectionContext::default();
    let mut rows = Vec::new();

    for line in normalized.lines() {
        ctx.update(line);
        if let Some(row) = parse_row(line, &ctx, &reg) {
            rows.push(row);
        }
    }

    rows
}

fn row<'a>(rows: &'a [ParsedRow], id: &str) -> &'a ParsedRow {
    rows.iter()
        .find(|row| row.analyte_id.as_deref() == Some(id))
        .unwrap_or_else(|| panic!("missing row {id}; parsed rows: {rows:#?}"))
}

#[test]
fn realistic_extracted_report_text_parses_header_rows_and_priors() {
    let text = "\
Exma Sra.
MARIA JOSE TESTE
Idade 39 Anos
Data de colheita 03/06/2024
Data de emissão 04/06/2024
Nº Inscrição ABC-123
Nº Processo PROC-9
Nº Origem ORIG-7
Entidade SNS
Requisitado por Dra. Exemplo
HEMATOLOGIA Resultado Valores de Referência Resultados anteriores 01/05/2024 01/04/2024
Eritrograma
Hemoglobina 13.4 g/dl 12.0 - 15.0 12.6 12.9
Eritrócitos 4.19 x 106/µl 3.80 - 4.80 4.02 4.13
Leucograma
Neutrófilos 61.3 % 5.02 x 103
/ 40.00 - 80.00 60.5 61.4
PATOLOGIA QUÍMICA
METABOLISMO LÍPIDICO
Colesterol LDL 101 mg/dL < 115
Baixo ou moderado: < 115 mg/dL
METABOLISMO DO FERRO
Ferritina 215 ng/ml 30 - 340
ENDOCRINOLOGIA
Tireoestimulina (TSH) 2.150 mUI/L 0.270 - 4.200
Vitamina D (25-OH) 39 ng/mL";

    let h = header::extract(text);
    assert_eq!(h.patient_name.as_deref(), Some("MARIA JOSE TESTE"));
    assert_eq!(h.inferred_sex, Some("f"));
    assert_eq!(h.collection_date_iso.as_deref(), Some("2024-06-03"));
    assert_eq!(h.emission_date_iso.as_deref(), Some("2024-06-04"));
    assert_eq!(h.inscription_id.as_deref(), Some("ABC-123"));
    assert_eq!(h.process_id.as_deref(), Some("PROC-9"));
    assert_eq!(h.origin_id.as_deref(), Some("ORIG-7"));
    assert_eq!(h.clinic_entity.as_deref(), Some("SNS"));
    assert_eq!(h.requesting_physician.as_deref(), Some("Dra. Exemplo"));

    let rows = parse_report_text(text);
    assert_eq!(rows.len(), 7, "{rows:#?}");

    let hemoglobina = row(&rows, "hemoglobina");
    assert_eq!(hemoglobina.value.numeric, Some(13.4));
    assert_eq!(hemoglobina.unit.as_deref(), Some("g/dL"));
    assert_eq!(hemoglobina.ref_low, Some(12.0));
    assert_eq!(hemoglobina.ref_high, Some(15.0));
    assert_eq!(
        hemoglobina.inline_priors,
        vec![
            ("2024-05-01".to_string(), 12.6),
            ("2024-04-01".to_string(), 12.9)
        ]
    );

    let neutrofilos = row(&rows, "neutrofilos");
    assert_eq!(neutrofilos.value.numeric, Some(61.3));
    assert_eq!(neutrofilos.unit.as_deref(), Some("%"));
    assert_eq!(neutrofilos.ref_low, Some(40.0));
    assert_eq!(neutrofilos.ref_high, Some(80.0));
    assert_eq!(neutrofilos.inline_priors.len(), 2);

    let ldl = row(&rows, "colesterol_ldl");
    assert_eq!(ldl.ref_grammar, RangeGrammar::Lt);
    assert_eq!(ldl.ref_high, Some(115.0));
    assert_eq!(ldl.flag, Some("normal"));

    let vitamina_d = row(&rows, "vitamina_d_25oh");
    assert_eq!(vitamina_d.value.numeric, Some(39.0));
    assert_eq!(vitamina_d.ref_grammar, RangeGrammar::None);
    assert!(vitamina_d.confidence > 0.99);
}

#[test]
fn categorical_reference_commentary_is_not_promoted_to_measurement_rows() {
    let rows = parse_report_text(
        "\
PATOLOGIA QUÍMICA
METABOLISMO LÍPIDICO
Colesterol LDL 101 mg/dL < 115
Baixo ou moderado: < 115 mg/dL
Elevado: <100 mg/dL
Muito elevado: < 70 mg/dL
Ferritina: <30 ng/mL
Ferritina 215 ng/ml 30 - 340",
    );

    assert_eq!(rows.len(), 2, "{rows:#?}");
    assert!(rows
        .iter()
        .any(|row| row.analyte_id.as_deref() == Some("colesterol_ldl")));
    assert!(rows
        .iter()
        .any(|row| row.analyte_id.as_deref() == Some("ferritina")));
}
