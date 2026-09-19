/** Portuguese (Portugal) catalogue for the data-heavy route pages. */
export const dataRoutesPtPT: Record<string, string> = {
  "Every patient with reports in this vault. Edit metadata, merge duplicates, or pre-create patients before their first PDF arrives.":
    "Todos os pacientes com relatórios neste cofre. Edite os metadados, una duplicados ou crie pacientes antes de chegar o primeiro PDF.",
  "Re-inferring…": "A inferir novamente…",
  "Re-infer sex": "Inferir sexo novamente",
  "Seeding…": "A preparar…",
  "Seed DOB": "Preencher data de nascimento",
  "+ New patient": "+ Novo paciente",
  "Last 12 months": "Últimos 12 meses",
  "Active patients": "Pacientes ativos",
  "No patients yet.": "Ainda não existem pacientes.",
  "Either ingest a PDF on the": "Importe um PDF no separador",
  "Ingest tab": "Importar",
  "or click": "ou clique em",
  "to create one manually.": "para criar um manualmente.",
  Filters: "Filtros",
  "Showing {shown} of {total}": "A mostrar {shown} de {total}",
  "Clear filters": "Limpar filtros",
  "Name / id": "Nome / id",
  Nickname: "Nome curto",
  Sex: "Sexo",
  "Has reports": "Tem relatórios",
  "Has notes": "Tem notas",
  "Has nickname": "Tem nome curto",
  "Recent activity": "Atividade recente",
  "Min reports": "Mín. de relatórios",
  "Min abnormal flags": "Mín. de alertas anómalos",
  "Min critical flags": "Mín. de alertas críticos",
  years: "anos",
  Any: "Qualquer",
  Male: "Masculino",
  Female: "Feminino",
  Other: "Outro",
  "Active in last 12 months": "Ativo nos últimos 12 meses",
  Yes: "Sim",
  No: "Não",
  Patient: "Paciente",
  DOB: "Data nasc.",
  Actions: "Ações",
  Latest: "Mais recente",
  "Most recent report collection date.":
    "Data de recolha do relatório mais recente.",
  "Number of ingested PDF reports for this patient.":
    "Número de relatórios PDF importados para este paciente.",
  'Distinct analytes measured directly on this patient\'s reports — values surfaced via the "Resultados anteriores" inline-prior columns are NOT counted.':
    "Analitos distintos medidos diretamente nos relatórios deste paciente — os valores das colunas anteriores não são contabilizados.",
  "Result rows flagged low / high / abnormal_qual across all reports (excludes inline priors).":
    "Linhas de resultados assinaladas como baixas / altas / anómalas em todos os relatórios (exclui valores anteriores).",
  "Result rows flagged critical_low / critical_high (excludes inline priors).":
    "Linhas de resultados assinaladas como criticamente baixas / altas (exclui valores anteriores).",
  "Edit sex on the patient detail page (or via the row's edit action).":
    "Edite o sexo na página de detalhe do paciente ou através da ação de edição da linha.",
  "Open patient detail": "Abrir detalhe do paciente",
  Open: "Abrir",
  Notes: "Notas",
  "Edit clinical notes": "Editar notas clínicas",
  "Add clinical notes": "Adicionar notas clínicas",
  "Edit name, nickname, sex, DOB":
    "Editar nome, nome curto, sexo e data de nascimento",
  Edit: "Editar",
  "Merge this patient into another (e.g. legal name change)":
    "Unir este paciente a outro (por exemplo, alteração do nome legal)",
  Merge: "Unir",
  "Delete patient and all reports": "Eliminar paciente e todos os relatórios",
  Delete: "Eliminar",
  "Editing patient": "A editar paciente",
  "Patient slug — derived from the display name":
    "Identificador do paciente — derivado do nome apresentado",
  "Display name": "Nome apresentado",
  "As printed on the lab PDF.": "Conforme aparece no PDF do laboratório.",
  optional: "opcional",
  "Friendly label shown in lists & titles.":
    "Nome amigável apresentado nas listas e títulos.",
  "Used for every flag derivation across this patient's reports.":
    "Usado para determinar todos os alertas nos relatórios deste paciente.",
  "Date of birth": "Data de nascimento",
  "save · cancel": "guardar · cancelar",
  "Save changes": "Guardar alterações",
  "Close dialog": "Fechar diálogo",
  "New patient": "Novo paciente",
  "Markdown not rendered — plain text only":
    "Markdown não é renderizado — apenas texto simples",
  "Merge into…": "Unir a…",
  "Merge into which patient?": "A que paciente pretende unir?",
  "No notes yet — click": "Ainda não existem notas — clique em",
  "to capture allergies, family history, ongoing conditions, etc.":
    "para registar alergias, antecedentes familiares, doenças contínuas, etc.",
  Report: "Relatório",
  "Merge patients": "Unir pacientes",
  "Delete this row?": "Eliminar esta linha?",
  "Delete row": "Eliminar linha",
  "Bulk delete": "Eliminar em massa",
  "Re-parse all": "Analisar tudo novamente",
  "Re-run the current parser against every stored report":
    "Executar novamente o analisador atual em todos os relatórios guardados",
  "Re-infer patient sex from each patient's stored raw_text — fixes legacy data ingested before the sex field was kept up to date.":
    "Inferir novamente o sexo a partir do texto original guardado — corrige dados antigos importados antes de o campo ser atualizado.",
  Refresh: "Atualizar",
  "Filter reports below": "Filtrar relatórios abaixo",
  "No reports match.": "Nenhum relatório corresponde.",
  "Select all visible": "Selecionar todos os visíveis",
  Sources: "Fontes",
  Rows: "Linhas",
  Conf: "Conf.",
  Compare: "Comparar",
  "Clear picks, filters, and any per-card overrides. Patient selection stays.":
    "Limpar seleções, filtros e substituições dos cartões. A seleção do paciente mantém-se.",
  "Date range": "Intervalo de datas",
  From: "De",
  Until: "Até",
  "Clip to the date span where every selected analyte has at least one reading.":
    "Limitar ao intervalo em que cada analito selecionado tem pelo menos uma leitura.",
  "Value bounds": "Limites dos valores",
  "Flag filter": "Filtro de alertas",
  "Normal only": "Apenas normais",
  Unflagged: "Sem alerta",
  "Reading source": "Origem das leituras",
  'Older values printed on newer reports\' "previous" columns. Off keeps things to first-class readings.':
    "Valores antigos impressos nas colunas “anteriores” de relatórios recentes. Desativado mantém apenas leituras principais.",
  "Keep last N per analyte (0 = all)":
    "Manter os últimos N por analito (0 = todos)",
  "Exclude specific reports": "Excluir relatórios específicos",
  "HRT anchor": "Referência de TH",
  "No HRT anchor set for this patient. Set":
    "Não existe uma referência de TH definida para este paciente. Defina",
  Presets: "Predefinições",
  "loading…": "a carregar…",
  "This preset also sets filters": "Esta predefinição também configura filtros",
  "Filter analytes": "Filtrar analitos",
  selected: "selecionados",
  "No analytes match that filter.": "Nenhum analito corresponde a esse filtro.",
  "Pick a patient to begin.": "Selecione um paciente para começar.",
  "Drag to reorder": "Arrastar para reordenar",
  "Move up": "Mover para cima",
  "Move down": "Mover para baixo",
  reading: "leitura",
  readings: "leituras",
  custom: "personalizado",
  global: "global",
  "Discard this chart's local overrides and snapshot the current global defaults again.":
    "Descartar as substituições locais deste gráfico e voltar a copiar as predefinições globais atuais.",
  "Choose which patient's readings to display. Trends are always patient-scoped — values from different patients are never combined into the same line.":
    "Escolha as leituras do paciente a apresentar. As tendências são sempre específicas do paciente — valores de pacientes diferentes nunca são combinados na mesma linha.",
  "Search patients…": "Pesquisar pacientes…",
  "Search patients": "Pesquisar pacientes",
  Patients: "Pacientes",
  "Patients with readings for this analyte":
    "Pacientes com leituras deste analito",
  "No patients match.": "Nenhum paciente corresponde.",
  "Export CSV": "Exportar CSV",
  "Download all readings of this analyte as CSV":
    "Transferir todas as leituras deste analito como CSV",
  "Open this code in the default browser":
    "Abrir este código no navegador predefinido",
  "Showing readings for": "A mostrar leituras de",
  "Also known as:": "Também conhecido como:",
  "SECTION HEADER": "CABEÇALHO DE SECÇÃO",
  "Not a measurable test": "Não é um teste mensurável",
  "is a heading the lab prints on the PDF":
    "é um cabeçalho que o laboratório imprime no PDF",
  "to introduce a group of related tests — it does not have its own value.":
    "para introduzir um grupo de testes relacionados — não tem um valor próprio.",
  "The actual measured analytes for this panel are listed below.":
    "Os analitos efetivamente medidos neste painel estão listados abaixo.",
  "The analyte Library powering every flag derivation, reference band, and chart context card.":
    "A Biblioteca de analitos que sustenta todos os alertas, bandas de referência e cartões de contexto dos gráficos.",
  "The Library has moved. Redirecting…":
    "A Biblioteca mudou de localização. A redirecionar…",
  "Select an analyte from the list to inspect its full Library entry.":
    "Selecione um analito da lista para consultar a entrada completa na Biblioteca.",
  "present in the Library so the parser can recognise this line on the PDF and skip it. It does not carry its own value; the analyte page renders an info-only card pointing to the panel’s measurable members.":
    "presente na Biblioteca para que o analisador reconheça esta linha no PDF e a ignore. Não tem um valor próprio; a página do analito mostra um cartão informativo com os elementos mensuráveis do painel.",
  "Reload the analyte Library from the bundled seed?\n\n• Re-installs every seed-bundled analyte's descriptions, reference ranges, categorical tiers, and aliases — your edits to seed entries will be lost.\n• User-created analytes and user-added aliases are preserved.\n• Existing parsed results are not touched.":
    "Recarregar a Biblioteca de analitos a partir da origem incluída?\n\n• Reinstala as descrições, intervalos de referência, níveis categóricos e aliases de todos os analitos incluídos — as suas alterações serão perdidas.\n• Os analitos criados pelo utilizador e os aliases adicionados pelo utilizador são preservados.\n• Os resultados já analisados não são alterados.",
  'Reload the analyte Library from the bundled seed?\n\n• Re-installs every seed-bundled analyte\'s descriptions, reference ranges, categorical tiers, and aliases — your edits to seed entries will be lost.\n• User-created analytes (source=user) and user-added aliases are preserved.\n• Existing parsed results are not touched. Run "Re-parse all" on Records afterwards if you want stored rows to pick up new Library fields.':
    'Recarregar a Biblioteca de analitos a partir da origem incluída?\n\n• Reinstala as descrições, intervalos de referência, níveis categóricos e aliases de todos os analitos incluídos — as suas alterações serão perdidas.\n• Os analitos criados pelo utilizador e os aliases adicionados pelo utilizador são preservados.\n• Os resultados já analisados não são alterados. Execute "Analisar tudo novamente" em Registos depois se quiser que as linhas guardadas recebam os novos campos da Biblioteca.',
  "Library fixes": "correções na Biblioteca",
  "The parser couldn't link these names to entries in the analyte Library. Click Link… to map a raw name to an existing analyte (creates a user alias and re-links every row that matches).":
    "O analisador não conseguiu associar estes nomes a entradas da Biblioteca de analitos. Clique em Associar… para mapear um nome original para um analito existente (cria um alias do utilizador e associa novamente todas as linhas correspondentes).",
  "{count} reading on file": "{count} leitura registada",
  "{count} readings on file": "{count} leituras registadas",
  "Section headers exist in the ontology so the parser can recognise the line on the PDF and skip it. They are informational only.":
    "Os cabeçalhos de secção existem na ontologia para que o analisador reconheça e ignore a linha no PDF. Servem apenas para informação.",
  "Section headers exist in the Library so the parser can recognise the line on the PDF and skip it. They are informational only.":
    "Os cabeçalhos de secção existem na Biblioteca para que o analisador reconheça e ignore a linha no PDF. Servem apenas para informação.",
  "Analyte {id} is not in the ontology. The values still display from your reports, but no clinical context is available.":
    "O analito {id} não existe na ontologia. Os valores continuam visíveis nos seus relatórios, mas não existe contexto clínico disponível.",
  "Analyte {id} is not in the Library. The values still display from your reports, but no clinical context is available.":
    "O analito {id} não existe na Biblioteca. Os valores continuam visíveis nos seus relatórios, mas não existe contexto clínico disponível.",
  Readings: "Leituras",
  of: "de",
  min: "mín.",
  reference: "referência",
  male: "masculino",
  female: "feminino",
  "Include {count} value from inline-prior columns of newer reports":
    "Incluir {count} valor das colunas de valores anteriores de relatórios mais recentes",
  "Include {count} values from inline-prior columns of newer reports":
    "Incluir {count} valores das colunas de valores anteriores de relatórios mais recentes",
  "{count} hidden": "{count} ocultos",
  "{kept} of {before} readings kept across {charts} chart.":
    "{kept} de {before} leituras mantidas em {charts} gráfico.",
  "{kept} of {before} readings kept across {charts} charts.":
    "{kept} de {before} leituras mantidas em {charts} gráficos.",
  "{count} hidden by the active filters.":
    "{count} ocultos pelos filtros ativos.",
  "none for this patient.": "nenhum para este paciente.",
  "{count} for this patient": "{count} para este paciente",
  "Reading from a per-chart preference snapshot — toolbar toggles here won't affect the global default.":
    "A ler uma cópia de preferências deste gráfico — os controlos aqui não alteram a predefinição global.",
  "Reading from the global default. Tick to give this chart its own isolated copy.":
    "A ler a predefinição global. Marque para dar a este gráfico uma cópia isolada.",
  "Shared timeline": "Linha temporal partilhada",
  intersection: "interseção",
  "Panel:": "Painel:",
  "No sibling analytes resolved for this panel yet.":
    "Ainda não foram resolvidos analitos irmãos para este painel.",
  "What it measures": "O que mede",
  "When elevated": "Quando elevado",
  "When reduced": "Quando reduzido",
  "Reference range & units": "Intervalo de referência e unidades",
  "No clinical context populated": "Sem contexto clínico preenchido",
  "Reload ontology": "Recarregar ontologia",
  "This usually means the seed file was updated after this DB was first set up. Click":
    "Isto normalmente significa que o ficheiro inicial foi atualizado depois de esta base de dados ter sido criada. Clique em",
  "to re-install.": "para reinstalar.",
  "Reference tiers": "Níveis de referência",
  "(current)": "(atual)",
  "No readings yet for this analyte.":
    "Ainda não existem leituras para este analito.",
  First: "Primeiro",
  Min: "Mín.",
  Max: "Máx.",
  Mean: "Média",
  "across {count} readings": "em {count} leituras",
  "Reference source": "Origem da referência",
  "Time elapsed since the previous reading for this patient. d = days, w = weeks, mo = months, y = years.":
    "Tempo desde a leitura anterior deste paciente. d = dias, s = semanas, m = meses, a = anos.",
  Value: "Valor",
  Unit: "Unidade",
  Flag: "Alerta",
  Method: "Método",
  Source: "Origem",
  prior: "anterior",
  "Clinical notes": "Notas clínicas",
  Reports: "Relatórios",
  "Recent source documents and review status.":
    "Documentos de origem recentes e estado de revisão.",
  "Filter reports…": "Filtrar relatórios…",
  "Filter patient reports": "Filtrar relatórios do paciente",
  "No reports yet for this patient.":
    "Ainda não existem relatórios para este paciente.",
  Signals: "Sinais",
  "Review-worthy analytes from the latest history.":
    "Analitos que merecem revisão no histórico mais recente.",
  "Filter analyte signals": "Filtrar sinais dos analitos",
  "Search analytes…": "Pesquisar analitos…",
  "Search flagged analytes": "Pesquisar analitos com alertas",
  "Loading analyte signals…": "A carregar sinais dos analitos…",
  "No analytes match this filter.": "Nenhum analito corresponde a este filtro.",
  Trends: "Tendências",
  "Most measured analytes and largest changes.":
    "Analitos mais medidos e maiores alterações.",
  Window: "Janela",
  "Trend time window": "Janela temporal das tendências",
  "Main analytes": "Analitos principais",
  "No numeric history yet.": "Ainda não existe histórico numérico.",
  "Biggest deltas": "Maiores variações",
  "Need at least two numeric readings.":
    "São necessárias pelo menos duas leituras numéricas.",
  "Open the full longitudinal analyte history":
    "Abrir o histórico longitudinal completo do analito",
  Prev: "Anterior",
  Next: "Seguinte",
  "Open the original PDF in your default viewer":
    "Abrir o PDF original no visualizador predefinido",
  "Open PDF": "Abrir PDF",
  "Download every parsed row as CSV":
    "Transferir todas as linhas analisadas como CSV",
  "Total rows": "Total de linhas",
  Matched: "Associadas",
  Unmatched: "Não associadas",
  "Inline priors": "Valores anteriores",
  "Avg confidence": "Confiança média",
  "Report metadata": "Metadados do relatório",
  collection: "recolha",
  emission: "emissão",
  requesting: "requisitante",
  inscription: "inscrição",
  "cycle phase": "fase do ciclo",
  "Used to pick the right reference range for cycle-dependent analytes (Estradiol, FSH, LH).":
    "Usado para escolher o intervalo de referência correto para analitos dependentes da fase do ciclo (estradiol, FSH, LH).",
  "— unknown —": "— desconhecida —",
  "Follicular (~days 1–13)": "Folicular (~dias 1–13)",
  "Ovulation (~day 14)": "Ovulação (~dia 14)",
  "Luteal (~days 15–28)": "Lútea (~dias 15–28)",
  "Post-menopause": "Pós-menopausa",
  Annotations: "Anotações",
  "Show previous values from inline-prior columns":
    "Mostrar valores anteriores das colunas de valores anteriores",
  Analyte: "Analito",
  Trend: "Tendência",
  Ref: "Ref.",
  "Printed in PDF as": "Impresso no PDF como",
  Add: "Adicionar",
  "Add a passkey or enable the native OS vault before removing the password.":
    "Adicione uma chave de acesso ou ative o cofre nativo do sistema antes de remover a palavra-passe.",
  "Age ≤": "Idade ≤",
  "Age ≥": "Idade ≥",
  "All four off = no flag filter. Multiple checks are unioned (e.g. abnormal + critical shows everything outside the normal range).":
    "As quatro opções desativadas = sem filtro de alertas. Várias opções são combinadas (por exemplo, anómalo + crítico mostra tudo fora do intervalo normal).",
  "All picks, filters, and overrides cleared.":
    "Todas as seleções, filtros e substituições foram limpos.",
  "Always use the analyte ontology": "Usar sempre a ontologia dos analitos",
  "Always use the lab's printed range":
    "Usar sempre o intervalo impresso pelo laboratório",
  "Anchor:": "Referência:",
  'Anchors the HRT timeline. Each report gets a "Day N / Month N / Year N" milestone measured from this date. Leave blank to hide the timeline section.':
    'Define a referência da linha temporal da TH. Cada relatório recebe um marco "Dia N / Mês N / Ano N" contado a partir desta data. Deixe vazio para ocultar a secção da linha temporal.',
  "Annotations saved": "Anotações guardadas",
  "Applies the same numeric cutoff to every selected analyte. Useful for clipping outliers on a single-unit panel; less useful when comparing analytes with different magnitudes.":
    "Aplica o mesmo limite numérico a todos os analitos selecionados. Útil para cortar valores extremos num painel de uma só unidade; menos útil ao comparar analitos com magnitudes diferentes.",
  "Cancel (Esc)": "Cancelar (Esc)",
  "Clear Compare state": "Limpar estado da Comparação",
  "Clear every picked analyte, every filter, and every per-card override?\n\nThe current patient selection stays.":
    "Limpar todos os analitos selecionados, todos os filtros e todas as substituições por cartão?\n\nA seleção atual do paciente será mantida.",
  "Click to change — sex is the global truth driving every flag derivation. Edit it through the patient metadata form.":
    "Clique para alterar — o sexo é a referência global usada para determinar todos os alertas. Edite-o através do formulário de metadados do paciente.",
  "Comma- or space-separated report IDs. Every reading sourced from these reports is dropped from every chart in the comparison. Useful for redacting a redo / unreliable draw without deleting it.":
    "IDs de relatórios separados por vírgulas ou espaços. Todas as leituras destes relatórios são excluídas de todos os gráficos da comparação. Útil para ocultar uma repetição ou colheita pouco fiável sem a eliminar.",
  "Compare reset": "Comparação reposta",
  "Context specific to this draw — e.g. 'fasting violated', 'first labs after starting estradiol valerate', 'redrawn after lab error'…":
    "Contexto específico desta colheita — por exemplo, 'jejum não cumprido', 'primeiras análises após iniciar valerato de estradiol', 'nova colheita após erro laboratorial'…",
  "Couldn’t load analyte info":
    "Não foi possível carregar a informação do analito",
  "Cycle phase updated": "Fase do ciclo atualizada",
  "Delete patient": "Eliminar paciente",
  'Delete patient "{name}" and ALL {count} report(s)?\n\nThis cannot be undone.':
    'Eliminar o paciente "{name}" e TODOS os {count} relatório(s)?\n\nEsta ação não pode ser anulada.',
  "Delete report": "Eliminar relatório",
  "Delete report {date} for {name}?":
    "Eliminar o relatório de {date} de {name}?",
  "Delete report {date} for {name}?\n\nThis removes the report, all {rows} parsed rows, and the cached PDF. This cannot be undone.":
    "Eliminar o relatório de {date} de {name}?\n\nIsto remove o relatório, as {rows} linhas analisadas e o PDF em cache. Esta ação não pode ser anulada.",
  "Delete report {date}?": "Eliminar o relatório de {date}?",
  "Delete {count} report(s)?\n\nAll parsed rows and cached PDFs will be removed. This cannot be undone.":
    "Eliminar {count} relatório(s)?\n\nTodas as linhas analisadas e os PDFs em cache serão removidos. Esta ação não pode ser anulada.",
  "Delete {count} selected": "Eliminar {count} selecionados",
  "Deleted patient {name}": "Paciente {name} eliminado",
  "Deleted {count} report(s)": "{count} relatório(s) eliminado(s)",
  "Deleted {deleted}, failed {failed}":
    "Eliminados: {deleted}; falharam: {failed}",
  "Deleting…": "A eliminar…",
  "Drives the calculated age column.": "Controla a coluna de idade calculada.",
  "Drives the calculated age in the header.":
    "Controla a idade calculada no cabeçalho.",
  "Dynamic preset — the analyte list is resolved per-patient at runtime":
    "Predefinição dinâmica — a lista de analitos é resolvida por paciente em tempo de execução",
  Exported: "Exportado",
  "Failed IDs: {ids}": "IDs que falharam: {ids}",
  Filtered: "Filtrados",
  "Free-form clinical context that doesn't fit the structured fields: allergies, family history, treatment plan, ongoing conditions.":
    "Contexto clínico livre que não cabe nos campos estruturados: alergias, antecedentes familiares, plano de tratamento e condições atuais.",
  "Free-form clinical context: allergies, family hx, treatment plan, ongoing conditions…":
    "Contexto clínico livre: alergias, antecedentes familiares, plano de tratamento e condições atuais…",
  "Hide filters": "Ocultar filtros",
  "If still empty after reloading, this analyte may not yet have descriptions in the bundled seed.":
    "Se continuar vazio depois de recarregar, este analito pode ainda não ter descrições na origem incluída.",
  "Include inline-prior values": "Incluir valores anteriores incorporados",
  "LASTNAME GIVEN NAME": "APELIDO NOME PRÓPRIO",
  "Language data downloaded": "Dados do idioma transferidos",
  "Library when usable, fall back to printed":
    "Biblioteca quando utilizável; caso contrário, usar o intervalo impresso",
  "Limits to patients with at least one report inside the rolling 12-month window.":
    "Limita aos pacientes com pelo menos um relatório no período móvel de 12 meses.",
  "Link…": "Associar…",
  "Local app reset": "Instância local reposta",
  "Low confidence": "Confiança baixa",
  "Manage patients, reports, and parsed data. Reports sharing the same date for the same patient are grouped — expand to see individual sources.":
    "Gira pacientes, relatórios e dados analisados. Os relatórios do mesmo paciente com a mesma data são agrupados — expanda para ver as fontes individuais.",
  'Merge "{name}" into…': 'Unir "{name}" a…',
  'Merge "{source}" INTO "{target}"?\n\nAll {count} report(s) will be reassigned to {target}, then {source} will be deleted. This cannot be undone.':
    'Unir "{source}" A "{target}"?\n\nOs {count} relatório(s) serão atribuídos a {target} e depois {source} será eliminado. Esta ação não pode ser anulada.',
  'Merge "{source}" INTO "{target}"?\n\nAll {count} report(s) will be reassigned to {target}, then {source} will be deleted. Useful for name changes (marriage / legal). This cannot be undone.':
    'Unir "{source}" A "{target}"?\n\nOs {count} relatório(s) serão atribuídos a {target} e depois {source} será eliminado. Útil para alterações de nome (casamento ou motivo legal). Esta ação não pode ser anulada.',
  'Merge "{source}" INTO "{target}"?\n\nAll {count} report(s) will be reassigned, then "{source}" will be deleted. This cannot be undone.':
    'Unir "{source}" A "{target}"?\n\nOs {count} relatório(s) serão reatribuídos e depois "{source}" será eliminado. Esta ação não pode ser anulada.',
  "Merge this patient into another (e.g. name change)":
    "Unir este paciente a outro (por exemplo, alteração do nome)",
  "Missing value": "Valor em falta",
  "Nickname saved": "Nome curto guardado",
  "No annotations on this report — click Add to capture context that's specific to this draw (deviations from protocol, recent meds, milestone moments, etc.). Patient-level notes live on the patient page.":
    "Não existem anotações neste relatório — clique em Adicionar para registar o contexto específico desta colheita (desvios ao protocolo, medicação recente, momentos importantes, etc.). As notas do paciente ficam na página do paciente.",
  "No filters active.": "Não existem filtros ativos.",
  "No patients": "Nenhum paciente",
  "No readings for {name}.": "Não existem leituras para {name}.",
  "No readings match the active filters for {name}.":
    "Nenhuma leitura de {name} corresponde aos filtros ativos.",
  "No reports match “{query}”.": "Nenhum relatório corresponde a “{query}”.",
  "Notes saved": "Notas guardadas",
  OCR: "OCR",
  "Patient DOB seeded": "Data de nascimento do paciente preenchida",
  "Patient already existed": "O paciente já existia",
  "Patient created": "Paciente criado",
  "Patient deleted": "Paciente eliminado",
  "Patient sex re-inferred": "Sexo do paciente inferido novamente",
  "Patient updated": "Paciente atualizado",
  "Patients merged": "Pacientes unidos",
  "Pre keeps readings strictly before this date; Post keeps the anchor day onwards.":
    "Pré mantém as leituras estritamente anteriores a esta data; Pós mantém o dia de referência e os dias seguintes.",
  "Pre-create a patient before their first PDF arrives. The next ingested report with the same name will UPSERT against this row, keeping the sex and DOB you set here.":
    "Crie um paciente antes de chegar o primeiro PDF. O próximo relatório importado com o mesmo nome atualizará esta linha, mantendo o sexo e a data de nascimento definidos aqui.",
  "Preferences reset": "Preferências repostas",
  "Re-infer sex from each patient's report raw_text — fixes legacy '?' values.":
    "Inferir novamente o sexo a partir do texto original de cada relatório — corrige valores '?' antigos.",
  "Re-parse all ({count})": "Analisar tudo novamente ({count})",
  "Re-parse all {count} reports with the current parser?\n\nDoesn't touch the source PDFs — just re-runs the parser against each report's stored raw text and replaces the parsed rows.":
    "Analisar novamente os {count} relatórios com o analisador atual?\n\nNão altera os PDFs de origem — executa novamente o analisador sobre o texto original guardado de cada relatório e substitui as linhas analisadas.",
  "Re-parse complete": "Nova análise concluída",
  "Re-parse done with {count} failures":
    "Nova análise concluída com {count} falhas",
  "Re-parsed": "Analisado novamente",
  "Re-parsing…": "A analisar novamente…",
  "Reload analyte ontology from the bundled seed?\n\n• Re-installs every seed-bundled analyte's descriptions, reference ranges, categorical tiers, and aliases — your edits to seed entries will be lost.\n• User-created analytes and user-added aliases are preserved.\n• Existing parsed results are not touched.":
    "Recarregar a ontologia dos analitos a partir da origem incluída?\n\n• Reinstala as descrições, intervalos de referência, níveis categóricos e aliases de todos os analitos incluídos — as suas alterações serão perdidas.\n• Os analitos criados pelo utilizador e os aliases adicionados pelo utilizador são preservados.\n• Os resultados já analisados não são alterados.",
  Rename: "Mudar nome",
  "Rename report": "Mudar nome do relatório",
  "Report deleted": "Relatório eliminado",
  "Report nickname saved": "Nome curto do relatório guardado",
  "Resolved at runtime": "Resolvido em tempo de execução",
  "Row deleted": "Linha eliminada",
  "Rows ({shown} of {total})": "Linhas ({shown} de {total})",
  "Save (⏎)": "Guardar (⏎)",
  "Search patients for this analyte": "Pesquisar pacientes deste analito",
  "Seed DOB for patients without one — uses literal 'Data de Nascimento' lines if present, else age + collection date.":
    "Preencher a data de nascimento dos pacientes sem uma — usa linhas literais 'Data de Nascimento' quando existem; caso contrário, usa a idade e a data de recolha.",
  "Select one or more analytes from the list, or apply a preset.":
    "Selecione um ou mais analitos da lista ou aplique uma predefinição.",
  "Select patient…": "Selecionar paciente…",
  "Select…": "Selecionar…",
  "Showing first 8 diagnostics; filter unmatched rows or re-parse after ontology fixes.":
    "A mostrar os primeiros 8 diagnósticos; filtre as linhas não associadas ou analise novamente depois de corrigir a ontologia.",
  "Showing first 8 diagnostics; filter unmatched rows or re-parse after Library fixes.":
    "A mostrar os primeiros 8 diagnósticos; filtre as linhas não associadas ou analise novamente depois de corrigir a Biblioteca.",
  "Stack multiple analytes for one patient on a shared timeline. Each plot has its own scale and reference band. Selections, filters, and per-card overrides persist across reloads.":
    "Sobreponha vários analitos de um paciente numa linha temporal partilhada. Cada gráfico tem a sua própria escala e banda de referência. As seleções, os filtros e as substituições por cartão persistem após recarregar.",
  "Tesseract OCR": "OCR Tesseract",
  "The analyte resolved ({id}) but its description, high/low meanings and unit notes are empty in the database.":
    "O analito {id} foi resolvido, mas a descrição, os significados de alto/baixo e as notas de unidade estão vazios na base de dados.",
  "The parser couldn't link these names to entries in the analyte ontology. Click Link… to map a raw name to an existing analyte (creates a user alias and re-links every row that matches).":
    "O analisador não conseguiu associar estes nomes a entradas da ontologia de analitos. Clique em Associar… para mapear um nome original para um analito existente (cria um alias do utilizador e associa novamente todas as linhas correspondentes).",
  "Time since the previous report ({date})":
    "Tempo desde o relatório anterior ({date})",
  "Unit mismatch": "Unidade incompatível",
  "Unmatched analyte": "Analito não associado",
  "Unparsed range": "Intervalo não analisado",
  'Updated metadata for "{name}".': 'Metadados de "{name}" atualizados.',
  "Clear patient filter": "Limpar filtro do paciente",
  "Map this raw name to a Library entry. The mapping persists as a user alias and":
    "Mapear este nome original para uma entrada da Biblioteca. O mapeamento persiste como alias do utilizador e",
  "Replace the current vault with a previous ZIP export?\n\n• The current vault will be renamed to data.backup-<timestamp> alongside the data dir.\n• The vault will be locked first; you will need the password from the source export to unlock it.\n• Restart the app after import for the new vault to take effect cleanly.\n\nThis cannot be undone via the UI — manual rollback only.":
    "Substituir o cofre atual por uma exportação ZIP anterior?\n\n• O cofre atual será renomeado para data.backup-<timestamp> junto à pasta de dados.\n• O cofre será bloqueado primeiro; precisará da palavra-passe da exportação de origem para o desbloquear.\n• Reinicie a aplicação depois da importação para aplicar o novo cofre corretamente.\n\nEsta ação não pode ser anulada através da aplicação — só é possível reverter manualmente.",
  annotated: "anotado",
  cleared: "limpo",
  conf: "conf.",
  "contains…": "contém…",
  draft: "rascunho",
  "e.g. Annual checkup, Pre-surgery panel…":
    "por exemplo, Consulta anual, Painel pré-cirúrgico…",
  "e.g. JOÃO PEDRO ALMEIDA": "por exemplo, JOÃO PEDRO ALMEIDA",
  "e.g. Mom, Dad, J.A.": "por exemplo, Mãe, Pai, J.A.",
  "e.g. Type 2 diabetes since 2018, on metformin 1000mg BD…":
    "por exemplo, Diabetes tipo 2 desde 2018, a tomar metformina 1000 mg 2x/dia…",
  "group(s)": "grupo(s)",
  lab: "laboratório",
  latest: "mais recente",
  loaded: "carregado",
  no: "não",
  "of {count}": "de {count}",
  "on the patient page to enable pre/post filtering here.":
    "na página do paciente para ativar aqui o filtro pré/pós.",
  resync: "sincronizar novamente",
  row: "linha",
  "save · cancel · empty input clears the nickname":
    "guardar · cancelar · deixar vazio remove o nome curto",
  "search…": "pesquisar…",
  "sets filters": "configura filtros",
  since: "desde",
  "since previous": "desde o anterior",
  source: "origem",
  "source(s)": "fonte(s)",
  "this patient": "este paciente",
  tier: "nível",
  "years old": "anos",
  "{count} analyte(s)": "{count} analito(s)",
  "{count} parse diagnostic(s)": "{count} diagnóstico(s) de análise",
  "{count} reading(s)": "{count} leitura(s)",
  "{count} report(s)": "{count} relatório(s)",
  "{count} report(s) moved to {target}.":
    "{count} relatório(s) movido(s) para {target}.",
  "{count} report(s) moved.": "{count} relatório(s) movido(s).",
  "{count} report(s) reassigned to {target}.":
    "{count} relatório(s) reatribuído(s) a {target}.",
  "{count} unmatched analyte(s)": "{count} analito(s) não associado(s)",
  "{name} delta trend": "Tendência de variação de {name}",
  "{name} longitudinal trend": "Tendência longitudinal de {name}",
  "{name} trend": "Tendência de {name}",
  "{rows} rows ({delta} vs before), {unmatched} unmatched, {diagnostics} diagnostics, conf {confidence}%":
    "{rows} linhas ({delta} vs. anterior), {unmatched} não associadas, {diagnostics} diagnósticos, conf. {confidence}%",
  "{succeeded}/{total} reports · {rows} total rows · {diagnostics} diagnostics":
    "{succeeded}/{total} relatórios · {rows} linhas no total · {diagnostics} diagnósticos",
  "{succeeded}/{total} succeeded. First failure: {id} — {error}":
    "{succeeded}/{total} concluídos. Primeira falha: {id} — {error}",
  "{updated}/{scanned} updated ({exact} exact · {approximate} approximate) · {unknown} still unknown":
    "{updated}/{scanned} atualizados ({exact} exatos · {approximate} aproximados) · {unknown} ainda desconhecidos",
  "{updated}/{scanned} updated · {unknown} still unknown":
    "{updated}/{scanned} atualizados · {unknown} ainda desconhecidos",
};
