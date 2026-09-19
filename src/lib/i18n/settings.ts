/** Portuguese (Portugal) translations for the Settings route. */
export const settingsPtPT: Record<string, string> = {
  // Settings navigation and appearance
  Settings: "Definições",
  "Settings sections": "Secções das definições",
  Appearance: "Aparência",
  "Theme & visual": "Tema e visual",
  Dashboard: "Painel",
  "Home layout": "Disposição da página inicial",
  Charts: "Gráficos",
  "Zoom, sliders, labels": "Zoom, controlos e etiquetas",
  Comparison: "Comparação",
  "Presets & filter sets": "Predefinições e filtros",
  Security: "Segurança",
  "OS vault & unlock": "Cofre do sistema e desbloqueio",
  Ingestion: "Importação",
  "PDF / OCR / LLM tiers": "Níveis de PDF / OCR / LLM",
  Ontology: "Ontologia",
  "Analyte registry": "Registo de analitos",
  Storage: "Armazenamento",
  "Paths & sizes": "Caminhos e tamanhos",
  About: "Sobre",
  "Build & environment": "Compilação e ambiente",
  Advanced: "Avançado",
  "Raw settings JSON": "JSON das definições brutas",
  Theme: "Tema",
  "Light, dark, or follow your OS preference.":
    "Claro, escuro ou seguir a preferência do sistema operativo.",
  Light: "Claro",
  Dark: "Escuro",
  System: "Sistema",
  Resolved: "Resolvido",
  "Language & regional format": "Idioma e formato regional",
  "Choose the app locale for dates, numbers, percentages, and browser language metadata. Automatic detection follows your device language and is the default.":
    "Escolha a localização da aplicação para datas, números, percentagens e metadados de idioma do navegador. A deteção automática segue o idioma do dispositivo e é a predefinição.",
  Locale: "Localização",
  "Application locale": "Localização da aplicação",
  "Active locale": "Localização ativa",
  "detected automatically": "detetada automaticamente",
  Automatic: "Automático",
  "Use your device language": "Usar o idioma do dispositivo",
  "English (US)": "Inglês (EUA)",
  "Português (Portugal)": "Português (Portugal)",
  "English regional formatting": "Formatação regional em inglês",
  "Portuguese regional formatting": "Formatação regional em português",
  "Accent color": "Cor de destaque",
  "Drives every interactive surface — buttons, links, focus rings, the primary chart series, the splash mark. Picks light + dark variants of the same hue automatically.":
    "Define todas as superfícies interativas — botões, ligações, anéis de foco, a série principal dos gráficos e a marca de arranque. Escolhe automaticamente variantes claras e escuras do mesmo tom.",
  "Accent: {label}": "Cor de destaque: {label}",
  Violet: "Violeta",
  Blue: "Azul",
  Emerald: "Esmeralda",
  Rose: "Rosa",
  Amber: "Âmbar",
  Teal: "Azul-petróleo",
  Slate: "Ardósia",
  Density: "Densidade",
  "Adjusts global card padding and line-height. Compact suits dense tables; comfortable gives every reading room to breathe.":
    "Ajusta o espaçamento global dos cartões e a altura das linhas. Compacto é adequado para tabelas densas; confortável dá espaço a cada leitura.",
  Comfortable: "Confortável",
  Compact: "Compacto",
  "Font size": "Tamanho do tipo de letra",
  "Scales the entire UI by adjusting the root font size. Tailwind text classes derive from this, so cards and tables scale together.":
    "Dimensiona toda a interface ajustando o tamanho de letra base. As classes de texto Tailwind derivam deste valor, por isso os cartões e as tabelas aumentam em conjunto.",
  Tiny: "Muito pequeno",
  Small: "Pequeno",
  Normal: "Normal",
  Large: "Grande",
  Gigantic: "Gigantesco",
  "Body font": "Tipo de letra do corpo",
  "The typeface for body copy. Numeric columns and chart axes always use a monospace regardless, so this only affects prose.":
    "O tipo de letra do texto corrido. As colunas numéricas e os eixos dos gráficos usam sempre uma fonte monoespaçada, pelo que esta opção afeta apenas a prosa.",
  "Sans (default)": "Sem serifa (predefinição)",
  Serif: "Com serifa",
  Mono: "Monoespaçada",
  Motion: "Movimento",
  "Reduce motion": "Reduzir movimento",
  "Disables splash animations, gradient blobs, and chart transitions. Honoured in addition to the system":
    "Desativa as animações de arranque, os gradientes e as transições dos gráficos. É respeitado em conjunto com a definição do sistema",
  "hint.": "de redução de movimento.",
  "Reset appearance": "Repor aparência",
  "Restores the violet / comfortable / normal / sans / automatic-locale defaults. Theme mode is unchanged.":
    "Repõe as predefinições violeta / confortável / normal / sem serifa / localização automática. O modo do tema não é alterado.",
  Reset: "Repor",

  // Dashboard
  "Dashboard layout": "Disposição do painel",
  "Choose which sections appear on the home dashboard and use the arrows to set their order. Preferences are stored locally with the rest of your encrypted app settings.":
    "Escolha as secções que aparecem no painel inicial e use as setas para definir a ordem. As preferências são guardadas localmente com as restantes definições encriptadas da aplicação.",
  "Reorder {label}": "Reordenar {label}",
  "Move up": "Mover para cima",
  "Move down": "Mover para baixo",
  "Move {label} up": "Mover {label} para cima",
  "Move {label} down": "Mover {label} para baixo",
  "Dashboard item limits": "Limites dos itens do painel",
  "These limits only change how many cards or rows are shown; the underlying records stay untouched.":
    "Estes limites alteram apenas o número de cartões ou linhas apresentados; os registos subjacentes permanecem intactos.",
  "Recent abnormal flags": "Classificações anómalas recentes",
  "Maximum patient cards in the spotlight.":
    "Número máximo de cartões de pacientes em destaque.",
  "Recent reports": "Relatórios recentes",
  "Newest imported reports listed on the dashboard.":
    "Relatórios importados mais recentemente apresentados no painel.",
  "Recent activity": "Atividade recente",
  "Latest audit events requested for the dashboard.":
    "Eventos de auditoria mais recentes pedidos pelo painel.",
  Patients: "Pacientes",
  "Patient cards shown after sorting by most recent activity.":
    "Cartões de pacientes apresentados após ordenar pela atividade mais recente.",
  "All patients": "Todos os pacientes",
  patients: "pacientes",
  "Reset dashboard": "Repor painel",
  "Summary cards": "Cartões de resumo",
  "Library-wide patient, report, abnormal, and critical totals.":
    "Totais globais de pacientes, relatórios, resultados anómalos e críticos.",
  "Patients with abnormal or critical results from the last 12 months.":
    "Pacientes com resultados anómalos ou críticos nos últimos 12 meses.",
  "The newest imported reports and their extraction confidence.":
    "Os relatórios importados mais recentes e a respetiva confiança de extração.",
  "The latest audit-log events from this vault.":
    "Os eventos mais recentes do registo de auditoria deste cofre.",
  "Patient cards ordered by most recent report activity.":
    "Cartões de pacientes ordenados pela atividade de relatório mais recente.",

  // Chart preferences
  "Chart interaction": "Interação com gráficos",
  "Auxiliary scrollers and wheel-zoom are off by default. Drag-to-pan inside the plot area always works, and the toolbar above each chart exposes the same toggles for one-off use.":
    "Os controlos de deslocamento auxiliares e o zoom com a roda estão desativados por predefinição. Arrastar dentro da área do gráfico funciona sempre e a barra acima de cada gráfico disponibiliza os mesmos controlos para uso pontual.",
  "Timeline range slider": "Controlo do intervalo temporal",
  "Adds a draggable horizontal stretcher under the chart for X-axis windowing.":
    "Adiciona um controlo horizontal arrastável sob o gráfico para ajustar a janela do eixo X.",
  "Vertical units slider": "Controlo das unidades verticais",
  "Adds a vertical stretcher on the right edge for Y-axis windowing.":
    "Adiciona um controlo vertical na margem direita para ajustar a janela do eixo Y.",
  "Mouse-wheel zoom inside chart": "Zoom com a roda dentro do gráfico",
  "Off by default — the wheel scrolls the page. Turn on to zoom the chart with the wheel (and Shift+wheel for the Y axis).":
    "Desligado por predefinição — a roda percorre a página. Ative para ampliar o gráfico com a roda (e Shift+roda para o eixo Y).",
  "Reference range source": "Origem do intervalo de referência",
  "Which range drives every flag pill, chart band, and reference card. Affects all analytes globally; the parser still captures every printed range regardless so you can switch back at any time without re-ingesting.":
    "Define qual intervalo alimenta cada classificação, banda do gráfico e cartão de referência. Afeta globalmente todos os analitos; o analisador continua a capturar todos os intervalos impressos, para que possa mudar de opção sem voltar a importar.",
  Auto: "Automático",
  Library: "Biblioteca",
  "Analyte Library": "Biblioteca de analitos",
  "Reload Library": "Recarregar biblioteca",
  "Library reloaded": "Biblioteca recarregada",
  "Always use the analyte Library": "Usar sempre a Biblioteca de analitos",
  "Per-report": "Por relatório",
  default: "predefinição",
  "library wins when the ontology has any usable reference (sex/cycle/tier/universal); falls back to the lab-printed range otherwise.":
    "a biblioteca prevalece quando a ontologia tem uma referência utilizável (sexo/ciclo/nível/universal); caso contrário, usa o intervalo impresso pelo laboratório.",
  "always derive from the analyte ontology — sex- and cycle-aware where applicable. Printed ranges from the PDF are ignored for flag computation.":
    "derivar sempre da ontologia do analito — considerando sexo e ciclo quando aplicável. Os intervalos impressos no PDF são ignorados ao calcular as classificações.",
  "trust whatever range the lab printed on each individual report, even when the ontology has a more specific default. Useful when your lab uses non-standard cutoffs you want to honour exactly.":
    "confiar no intervalo impresso pelo laboratório em cada relatório, mesmo quando a ontologia tem uma predefinição mais específica. Útil quando o laboratório usa limites não standard que pretende respeitar exatamente.",
  "the Library wins when it has any usable reference (sex/cycle/tier/universal); falls back to the lab-printed range otherwise.":
    "a Biblioteca é prioritária quando tem uma referência utilizável (sexo/ciclo/nível/universal); caso contrário, usa o intervalo impresso pelo laboratório.",
  "always derive from the analyte Library — sex- and cycle-aware where applicable. Printed ranges from the PDF are ignored for flag computation.":
    "derivar sempre da Biblioteca de analitos — considerando sexo e ciclo quando aplicável. Os intervalos do PDF são ignorados ao calcular alertas.",
  "trust whatever range the lab printed on each individual report, even when the Library has a more specific default. Useful when your lab uses non-standard cutoffs you want to honour exactly.":
    "confiar no intervalo impresso pelo laboratório em cada relatório, mesmo quando a Biblioteca tem uma predefinição mais específica. Útil quando o laboratório usa limites não convencionais que pretende respeitar exatamente.",
  "Default chart appearance": "Aparência predefinida dos gráficos",
  "Every toolbar toggle on a chart writes through to the corresponding default here, so what you set per-chart sticks app-wide. Reset to the bundled defaults with the button at the bottom.":
    "Cada controlo da barra de um gráfico atualiza aqui a predefinição correspondente, pelo que as alterações feitas num gráfico permanecem em toda a aplicação. Reponha as predefinições incluídas com o botão no fim.",
  "Symbols at each reading": "Símbolos em cada leitura",
  "Off → just the line. Useful for very dense series where every-point markers crowd the canvas.":
    "Desligado → apenas a linha. Útil para séries muito densas em que os marcadores de cada ponto ocupam demasiado espaço.",
  "Use report nicknames on the X axis":
    "Usar nomes curtos dos relatórios no eixo X",
  "When the source report has a nickname, replace the date with it. Falls back to the date for unlabelled reports.":
    "Quando o relatório de origem tem um nome curto, substitui a data por esse nome. Usa a data nos relatórios sem nome.",
  "Trend line": "Linha de tendência",
  "Overlay a dashed linear-regression line over the visible points.":
    "Sobrepor uma linha tracejada de regressão linear aos pontos visíveis.",
  "Today reference line": "Linha de referência de hoje",
  "Vertical dashed marker at today's date.":
    "Marcador vertical tracejado na data de hoje.",
  "Y-axis scale": "Escala do eixo Y",
  "Logarithmic clamps to": "A escala logarítmica muda para",
  "automatically when any value is ≤ 0.":
    "automaticamente quando algum valor é ≤ 0.",
  Linear: "Linear",
  log: "Logarítmica",
  "Date format": "Formato da data",
  "How dates appear on the X-axis when nicknames aren't being used.":
    "Define como as datas aparecem no eixo X quando não são usados nomes curtos.",
  "Grid density": "Densidade da grelha",
  "Off hides every gridline. Detailed adds minor ticks for fine-grained reading.":
    "Desligado oculta todas as linhas da grelha. Detalhado adiciona marcas secundárias para uma leitura fina.",
  Off: "Desligado",
  Standard: "Standard",
  Detailed: "Detalhado",
  "Line width": "Espessura da linha",
  "Stroke thickness in pixels.": "Espessura do traço em píxeis.",
  "Symbol size": "Tamanho do símbolo",
  "Diameter of the dot at each reading, in pixels.":
    "Diâmetro do ponto de cada leitura, em píxeis.",
  "Series rendering": "Apresentação da série",
  "How the data line is drawn. Bar mode disables smoothing and step interpolation (those only apply to line / area).":
    "Define como a linha de dados é desenhada. O modo de barras desativa a suavização e a interpolação por degraus (aplicáveis apenas a linha / área).",
  "Series type": "Tipo de série",
  "Line keeps the chart light; area fills the region beneath the line; bar shows discrete vertical bars.":
    "Linha mantém o gráfico leve; área preenche a região sob a linha; barras mostram barras verticais discretas.",
  Line: "Linha",
  Area: "Área",
  Bar: "Barras",
  "Step interpolation": "Interpolação por degraus",
  "None = straight (or smoothed) segments. Start / Middle / End render staircase steps anchored at the indicated edge of each interval — useful when readings represent values that hold steady between samples.":
    "Nenhum = segmentos retos (ou suavizados). Início / Meio / Fim desenham degraus ancorados na margem indicada de cada intervalo — útil quando as leituras representam valores estáveis entre amostras.",
  None: "Nenhum",
  Start: "Início",
  Middle: "Meio",
  End: "Fim",
  "Colour points by flag": "Colorir pontos pela classificação",
  "Tints each reading marker by its low / normal / high / critical flag, on top of the line colour.":
    "Colore cada marcador pela sua classificação baixa / normal / elevada / crítica, sobre a cor da linha.",
  "Connect across gaps": "Ligar através de falhas",
  "When off, missing readings break the line into segments; when on, the line bridges any null/undefined value.":
    "Quando desligado, as leituras em falta dividem a linha em segmentos; quando ligado, a linha transpõe valores null/undefined.",
  "Mean line": "Linha da média",
  "Horizontal dashed line at the mean of the visible readings.":
    "Linha horizontal tracejada na média das leituras visíveis.",
  "Min & max markers": "Marcadores mínimo e máximo",
  "Pin labels at the highest and lowest readings.":
    "Fixar etiquetas nas leituras mais alta e mais baixa.",
  "Reference band opacity": "Opacidade da banda de referência",
  "100% = bundled token alpha. 0% hides the fill entirely; 200% doubles its boldness.":
    "100% = alfa do token incluído. 0% oculta completamente o preenchimento; 200% duplica a intensidade.",
  "Animation speed": "Velocidade da animação",
  "Off disables every chart transition for instant rendering.":
    "Desligado desativa todas as transições dos gráficos para uma apresentação imediata.",
  Fast: "Rápida",
  Slow: "Lenta",
  "Tooltip & chrome": "Dicas e elementos do gráfico",
  "Tooltip mode": "Modo das dicas",
  "Axis = crosshair pointer with the nearest reading; Item = hover only the exact point underneath the cursor.":
    "Eixo = mira com a leitura mais próxima; item = passar apenas sobre o ponto exato sob o cursor.",
  "Axis (crosshair)": "Eixo (mira)",
  "Item (point)": "Item (ponto)",
  "Show chart titles": "Mostrar títulos dos gráficos",
  "When off, the chart's `title` prop is ignored — useful when the page already provides the heading.":
    "Quando desligado, a propriedade `title` do gráfico é ignorada — útil quando a página já fornece o título.",
  "X-axis label overflow": "Excesso de etiquetas no eixo X",
  "What to do when too many dates would collide on the time axis.":
    "Define o que fazer quando demasiadas datas colidem no eixo temporal.",
  "hides overlapping labels (date still on hover).":
    "oculta etiquetas sobrepostas (a data continua disponível ao passar o rato).",
  Rotate: "Rodar",
  "tilts each label 35° so more fit before clipping.":
    "inclina cada etiqueta 35° para caberem mais antes de serem cortadas.",
  Hide: "Ocultar",
  "never draws axis labels — cleanest visual; rely on hover for the exact date.":
    "nunca desenha etiquetas no eixo — visual mais limpo; passe o rato para ver a data exata.",
  "Default time window": "Janela temporal predefinida",
  "At chart open, narrow the visible range to the most recent N. Older readings are still on file and reachable via slider/wheel zoom-out — this just controls the first paint.":
    "Ao abrir o gráfico, limita o intervalo visível às N leituras mais recentes. As leituras antigas continuam disponíveis através do controlo ou do zoom de redução — isto controla apenas a apresentação inicial.",
  All: "Tudo",
  "30 days": "30 dias",
  "90 days": "90 dias",
  "6 months": "6 meses",
  "1 year": "1 ano",
  "2 years": "2 anos",
  "5 years": "5 anos",
  "Default zoom & axis padding": "Zoom e margem dos eixos predefinidos",
  'How much breathing room the axes leave around the data. The X axis is single-stage — the chart always opens flush against the data. The Y axis is two-stage: the "axis padding" is the maximum zoom-out, and the "default visible padding" is how snug it opens.':
    'Define o espaço que os eixos deixam em torno dos dados. O eixo X tem uma só fase — o gráfico abre sempre junto aos dados. O eixo Y tem duas fases: a "margem do eixo" é a redução máxima do zoom e a "margem visível predefinida" define quão ajustado abre.',
  "X-axis padding": "Margem do eixo X",
  "Tight margin past the first/last point on the time axis. 0% pins the line edge-to-edge.":
    "Margem reduzida além do primeiro/último ponto no eixo temporal. 0% encosta a linha às margens.",
  "Y-axis maximum padding": "Margem máxima do eixo Y",
  "How far past the data the user can zoom OUT vertically. Larger = more headroom for reference bands that extend beyond the data.":
    "Define quanto o utilizador pode reduzir verticalmente o zoom para além dos dados. Um valor maior dá mais espaço às bandas de referência que ultrapassam os dados.",
  "Y-axis default visible padding": "Margem visível predefinida do eixo Y",
  "How snug the chart opens vertically. Lower = data fills the plot at first paint.":
    "Define quão ajustado o gráfico abre verticalmente. Um valor menor faz os dados preencherem o gráfico desde o início.",
  "Reset chart defaults": "Repor predefinições dos gráficos",
  "Restore every chart preference on this page (and the toolbar toggles) to bundled defaults.":
    "Repõe todas as preferências de gráficos desta página (e os controlos da barra) nas predefinições incluídas.",

  // Comparison presets
  "Comparison presets": "Predefinições de comparação",
  "New preset": "Nova predefinição",
  "Reset bundled": "Repor incluídas",
  "Restore every bundled preset to its original definition. User presets are kept.":
    "Repõe cada predefinição incluída na definição original. As predefinições do utilizador são mantidas.",
  dynamic: "dinâmica",
  "Patient-aware: analyte list is filled at runtime.":
    "Ciente do paciente: a lista de analitos é preenchida durante a execução.",
  user: "utilizador",
  customised: "personalizada",
  bundled: "incluída",
  "Sets filter overrides on click": "Define substituições de filtros ao clicar",
  filters: "filtros",
  Edit: "Editar",
  Delete: "Eliminar",
  "Delete this preset": "Eliminar esta predefinição",
  "Restore bundled defaults": "Restaurar predefinições incluídas",
  "You can rename it and attach filters here.":
    "Pode mudar-lhe o nome e associar filtros aqui.",
  "Display name": "Nome apresentado",
  Analytes: "Analitos",
  "Search analytes to add…": "Pesquisar analitos para adicionar…",
  add: "adicionar",
  "No analytes match.": "Nenhum analito corresponde.",
  "Set filters when this preset is applied":
    "Definir filtros ao aplicar esta predefinição",
  "When off, applying the preset only changes the picked analytes; current filters stay as-is.":
    "Quando desligado, aplicar a predefinição altera apenas os analitos escolhidos; os filtros atuais mantêm-se.",
  "Date from": "Data inicial",
  "Date until": "Data final",
  "Value min": "Valor mínimo",
  "Value max": "Valor máximo",
  "Last N per analyte (0 = all)": "Últimas N leituras por analito (0 = todas)",
  "HRT anchor": "Referência de TH",
  "All readings": "Todas as leituras",
  "Pre-HRT": "Antes da TH",
  "Post-HRT": "Depois da TH",
  "Exclude report IDs (comma/space-separated)":
    "Excluir IDs de relatórios (separados por vírgulas/espaços)",
  "Include inline-prior": "Incluir resultados anteriores no relatório",
  "Intersection only": "Apenas interseção",
  "Normal only": "Apenas normais",
  Abnormal: "Anómalos",
  Critical: "Críticos",
  Unflagged: "Sem classificação",
  Cancel: "Cancelar",
  "Create preset": "Criar predefinição",
  "Save changes": "Guardar alterações",
  "e.g. Iron panel": "ex.: Painel de ferro",
  "e.g. Liver — last year only": "ex.: Fígado — apenas o último ano",

  // Security and authentication
  "Device security": "Segurança do dispositivo",
  "Manage the protection layers used by this local instance. The encrypted SQLCipher database and managed PDF cache remain protected whether or not OS-vault unlock is enabled.":
    "Gira as camadas de proteção usadas por esta instância local. A base de dados SQLCipher encriptada e a cache de PDFs gerida permanecem protegidas, quer o desbloqueio pelo cofre do sistema esteja ativo ou não.",
  "Data master key": "Chave mestre dos dados",
  Enabled: "Ativado",
  "Not initialized": "Não inicializado",
  "Database and PDF cache encryption key":
    "Chave de encriptação da base de dados e da cache de PDFs",
  "Native OS vault": "Cofre nativo do sistema operativo",
  "Not enabled": "Não ativado",
  Session: "Sessão",
  Unlocked: "Desbloqueada",
  Locked: "Bloqueada",
  "DMK held only in native memory": "DMK mantida apenas na memória nativa",
  "Disable OS vault": "Desativar cofre do sistema",
  "Automatic unlock": "Desbloqueio automático",
  "Unlock at launch when the OS credential store permits it.":
    "Desbloquear ao iniciar quando o armazenamento de credenciais do sistema o permitir.",
  "Enable native OS vault": "Ativar cofre nativo do sistema",
  "Enable this while unlocked to place a device-bound DMK copy in {platform}.":
    "Ative esta opção enquanto está desbloqueado para guardar uma cópia da DMK associada ao dispositivo em {platform}.",
  "Vault password": "Palavra-passe do cofre",
  "Change the password used to recover this encrypted vault.":
    "Altere a palavra-passe usada para recuperar este cofre encriptado.",
  "Add a password recovery method to this OS-vault-only instance.":
    "Adicione um método de recuperação por palavra-passe a esta instância protegida apenas pelo cofre do sistema.",
  "Current vault password": "Palavra-passe atual do cofre",
  "New vault password": "Nova palavra-passe do cofre",
  "Confirm new password": "Confirmar nova palavra-passe",
  "Saving…": "A guardar…",
  "Change password": "Alterar palavra-passe",
  "Add password": "Adicionar palavra-passe",
  "Remove password": "Remover palavra-passe",
  "Add a passkey or enable the native OS vault before removing the last recovery method.":
    "Adicione uma chave de acesso ou ative o cofre nativo do sistema antes de remover o último método de recuperação.",
  Passkeys: "Chaves de acesso",
  "Register WebAuthn authenticators such as Windows Hello, Touch ID, or a hardware key. They unlock locally through the authenticator PRF and never replace the encrypted vault.":
    "Registe autenticadores WebAuthn, como Windows Hello, Touch ID ou uma chave física. Desbloqueiam localmente através do PRF do autenticador e nunca substituem o cofre encriptado.",
  Passkey: "Chave de acesso",
  Remove: "Remover",
  "No passkeys are configured.": "Não existem chaves de acesso configuradas.",
  "Passkey label (optional)": "Etiqueta da chave de acesso (opcional)",
  "Registering…": "A registar…",
  "Add passkey": "Adicionar chave de acesso",
  "WebAuthn is unavailable in this WebView.":
    "WebAuthn não está disponível nesta WebView.",
  "Rotate data master key": "Rodar chave mestre dos dados",
  "Re-key the encrypted database, managed PDFs, native OS-vault copy, password wrapper, and registered passkey wrappers. Each configured passkey will ask for a fresh authenticator assertion.":
    "Volta a encriptar a base de dados, os PDFs geridos, a cópia no cofre nativo do sistema, o invólucro da palavra-passe e os invólucros das chaves de acesso registadas. Cada chave de acesso configurada pedirá uma nova asserção do autenticador.",
  "Your unlocked session authorizes this rotation; no password entry is required.":
    "A sua sessão desbloqueada autoriza esta rotação; não é necessária a introdução de uma palavra-passe.",
  "Rotating…": "A rodar…",
  "Loading security status…": "A carregar o estado de segurança…",
  "Protection model.": "Modelo de proteção.",
  "Passwords and passkeys continue to work as recovery methods. The OS vault is an additional device-local wrapper, not a replacement for the encrypted database.":
    "As palavras-passe e as chaves de acesso continuam a funcionar como métodos de recuperação. O cofre do sistema é um invólucro adicional, local ao dispositivo, e não substitui a base de dados encriptada.",
  "Shared-device warning.": "Aviso para dispositivos partilhados.",
  "Anyone who can unlock this operating-system account may be able to use the optional automatic unlock setting. Keep it off on shared or unattended machines.":
    "Qualquer pessoa que consiga desbloquear esta conta do sistema operativo poderá usar a opção de desbloqueio automático. Mantenha-a desligada em máquinas partilhadas ou sem supervisão.",
  "Security checklist": "Lista de verificação de segurança",
  "Use a strong vault password and keep at least one recovery method available.":
    "Use uma palavra-passe forte para o cofre e mantenha pelo menos um método de recuperação disponível.",
  "Lock the app when stepping away from an unlocked session.":
    "Bloqueie a aplicação quando se afastar de uma sessão desbloqueada.",
  "Export backups to a separately protected location; exported PDFs and the database remain encrypted.":
    "Exporte cópias de segurança para um local protegido separadamente; os PDFs exportados e a base de dados permanecem encriptados.",

  // Ingestion and models
  "Ingestion pipeline": "Pipeline de importação",
  "Choose which extraction stages are allowed when a PDF is imported. Tier 1 is the foundation; higher tiers are opt-in fallbacks for difficult documents.":
    "Escolha as fases de extração permitidas ao importar um PDF. O nível 1 é a base; os níveis superiores são alternativas opcionais para documentos difíceis.",
  "Tier 1": "Nível 1",
  "PDF extraction": "Extração de PDF",
  "Required foundation for every import.":
    "Base obrigatória para todas as importações.",
  "Tier 2": "Nível 2",
  "Run Tesseract when embedded PDF text is sparse.":
    "Executar o Tesseract quando o texto incorporado no PDF for escasso.",
  "Tier 3": "Nível 3",
  "Hybrid OCR + LLM": "OCR híbrido + LLM",
  "Repair low-confidence OCR with Phi-4 when the model is enabled and ready.":
    "Corrigir OCR de baixa confiança com o Phi-4 quando o modelo estiver ativo e pronto.",
  "Tier 1 is disabled; imports will fail closed until PDF extraction is enabled.":
    "O nível 1 está desativado; as importações falharão de forma segura até a extração de PDF ser ativada.",
  "Always-on baseline. Other tiers below are opt-in.":
    "Base sempre ativa. Os níveis seguintes são opcionais.",
  "Library bound. Ingestion ready.": "Biblioteca ligada. Importação pronta.",
  "Library missing — ingestion will fail.":
    "Biblioteca em falta — a importação irá falhar.",
  Estimated: "Estimado",
  "on disk": "no disco",
  OK: "OK",
  Missing: "Em falta",
  "Canceling…": "A cancelar…",
  "File {current} of {total}": "Ficheiro {current} de {total}",
  "Higher-tier extraction resources": "Recursos de extração de nível superior",
  "These resources support Tier 2 and Tier 3 only. Each loads only when used; compile with the relevant cargo feature to make a resource available.":
    "Estes recursos suportam apenas os níveis 2 e 3. Cada um é carregado apenas quando usado; compile com a funcionalidade Cargo correspondente para disponibilizar o recurso.",
  "Use Tesseract when a PDF has sparse or unusable embedded text.":
    "Usar o Tesseract quando um PDF tiver texto incorporado escasso ou inutilizável.",
  "no LLM": "sem LLM",
  compiled: "compilado",
  yes: "sim",
  "no — rebuild with --features tesseract-ocr":
    "não — recompile com --features tesseract-ocr",
  languages: "idiomas",
  present: "presente",
  missing: "em falta",
  runtime: "tempo de execução",
  ready: "pronto",
  unavailable: "indisponível",
  "Download eng data": "Transferir dados eng",
  "Download por data": "Transferir dados por",
  "Delete managed data": "Eliminar dados geridos",
  "Native binary guide": "Guia dos binários nativos",
  "Use optional repair and vision models for difficult or low-confidence results.":
    "Usar modelos opcionais de correção e visão para resultados difíceis ou de baixa confiança.",
  "repair tier": "nível de correção",
  model: "modelo",
  "no — rebuild with --features embedded-llm":
    "não — recompile com --features embedded-llm",
  loading: "a carregar",
  "Download {label} model": "Transferir modelo {label}",
  "Download the recommended Phi-4 Q4_K_M model (about 2.5 GB) into the encrypted app data folder?":
    "Transferir o modelo Phi-4 Q4_K_M recomendado (cerca de 2,5 GB) para a pasta de dados encriptada da aplicação?",
  "Download the complete olmOCR-2 7B model snapshot (about 16 GB) into the app data folder?":
    "Transferir o snapshot completo do modelo olmOCR-2 7B (cerca de 16 GB) para a pasta de dados da aplicação?",
  Download: "Transferir",
  "Downloading…": "A transferir…",
  "Download cancelled": "Transferência cancelada",
  "{label} was not installed.": "{label} não foi instalado.",
  "Model download complete": "Transferência do modelo concluída",
  "{label} is configured for this instance.":
    "{label} está configurado para esta instância.",
  "Tesseract {language} data is ready in the managed models folder.":
    "Os dados {language} do Tesseract estão prontos na pasta de modelos geridos.",
  "Tesseract {language} data was not installed.":
    "Os dados {language} do Tesseract não foram instalados.",
  "Choose Phi-4 model file": "Escolher ficheiro do modelo Phi-4",
  "Choose olmOCR model file": "Escolher ficheiro do modelo olmOCR",
  Browse: "Procurar",
  Load: "Carregar",
  Loading: "A carregar",
  Unload: "Descarregar",
  "vision OCR": "OCR de visão",
  "no — rebuild with --features embedded-ocr-vision":
    "não — recompile com --features embedded-ocr-vision",
  "set model_path in this tier": "defina model_path neste nível",
  "No model_path configured": "Nenhum model_path configurado",

  // Ontology
  "Analyte ontology": "Ontologia de analitos",
  "Re-installs analyte definitions (descriptions, categorical tiers, default reference ranges, aliases) from the bundled seed. Use after upgrading the app, then hit":
    "Reinstala as definições dos analitos (descrições, níveis categóricos, intervalos de referência predefinidos e aliases) a partir da origem incluída. Use depois de atualizar a aplicação e, em seguida, selecione",
  "Records → Re-parse all": "Registos → Analisar tudo novamente",
  "to refresh existing rows.": "para atualizar as linhas existentes.",
  "Reloading…": "A recarregar…",
  "Reload from seed": "Recarregar da origem",
  "Browse / edit": "Procurar / editar",
  "The full Ontology management tab — search, filter, view JSON, create / edit / delete user analytes — lives at":
    "O separador completo de gestão da ontologia — pesquisar, filtrar, ver JSON e criar / editar / eliminar analitos do utilizador — está em",
  "The full Library management tab — search, filter, view JSON, create / edit / delete user analytes — lives at":
    "O separador completo de gestão da Biblioteca — pesquisar, filtrar, ver JSON e criar / editar / eliminar analitos do utilizador — está em",
  "Open ontology browser →": "Abrir navegador da ontologia →",
  "Open Library browser →": "Abrir navegador da Biblioteca →",

  // Storage, import and export
  "Storage paths & sizes": "Caminhos e tamanhos do armazenamento",
  "Where data lives on this device. Everything is encrypted at rest.":
    "Onde os dados estão neste dispositivo. Tudo é encriptado em repouso.",
  "Open data folder": "Abrir pasta de dados",
  Refresh: "Atualizar",
  "Refreshing…": "A atualizar…",
  "Data dir": "Pasta de dados",
  total: "total",
  Database: "Base de dados",
  "WAL/journal": "WAL/diário",
  Keystore: "Armazém de chaves",
  "Encrypted PDF cache": "Cache de PDFs encriptada",
  file: "ficheiro",
  files: "ficheiros",
  "Models dir": "Pasta de modelos",
  "Database contents": "Conteúdo da base de dados",
  "A look inside the encrypted SQLite file.":
    "Uma vista do ficheiro SQLite encriptado.",
  Reports: "Relatórios",
  Results: "Resultados",
  "inline-prior": "resultado anterior no relatório",
  aliases: "aliases",
  "Audit entries": "Entradas de auditoria",
  "Date span": "Intervalo de datas",
  "Encryption engine": "Motor de encriptação",
  "Journal mode": "Modo de diário",
  "Page size": "Tamanho da página",
  "Page count": "Número de páginas",
  "On-disk pages": "Páginas no disco",
  "Vault locked. Unlock to see patient / report / result counts and SQLCipher diagnostics.":
    "Cofre bloqueado. Desbloqueie para ver os números de pacientes / relatórios / resultados e os diagnósticos SQLCipher.",
  "Backup & restore": "Cópia de segurança e restauro",
  "Export packages the entire encrypted vault — DB, keystore, PDFs, models — into a ZIP file in a folder of your choice. The files remain encrypted, so the package is safe to keep on a USB stick or sync to an external backup tool. Import restores from a previous ZIP export (the current vault is renamed aside, not deleted, so you can roll back).":
    "A exportação reúne todo o cofre encriptado — base de dados, armazém de chaves, PDFs e modelos — num ficheiro ZIP numa pasta à sua escolha. Os ficheiros permanecem encriptados, pelo que o pacote pode ser guardado num dispositivo USB ou sincronizado com uma ferramenta de cópia externa. A importação restaura uma exportação ZIP anterior (o cofre atual é renomeado, não eliminado, para permitir recuar).",
  "Choose a destination folder for the vault ZIP export":
    "Escolher uma pasta de destino para a exportação ZIP do cofre",
  "Export vault ZIP…": "Exportar ZIP do cofre…",
  "Exporting ZIP…": "A exportar ZIP…",
  "Import vault ZIP…": "Importar ZIP do cofre…",
  "Importing ZIP…": "A importar ZIP…",
  "Vault exported": "Cofre exportado",
  "Vault imported": "Cofre importado",
  "{count} files · {size} written to {destination}":
    "{count} ficheiros · {size} escritos em {destination}",
  "{count} files · {size} restored. Restart the app to continue.":
    "{count} ficheiros · {size} restaurados. Reinicie a aplicação para continuar.",
  "Last export": "Última exportação",
  "Last import — restart the app to use the imported vault.":
    "Última importação — reinicie a aplicação para usar o cofre importado.",
  "Restored from": "Restaurado de",
  "Previous vault preserved at": "Cofre anterior preservado em",
  "Export:": "Exportação:",
  "safe at any time. Audit-logged. A ZIP package is created in the selected directory; that directory must not be inside the vault.":
    "segura em qualquer momento. Registada na auditoria. É criado um pacote ZIP no diretório selecionado; esse diretório não pode estar dentro do cofre.",
  "Import:": "Importação:",
  "select a bloody-level ZIP and lock the vault first (the open SQLite handle would otherwise pin the old DB and corrupt the swap on Windows). The current vault is renamed to":
    "selecione um ZIP do bloody-level e bloqueie primeiro o cofre (caso contrário, o identificador SQLite aberto manteria a base de dados antiga presa e corromperia a troca no Windows). O cofre atual é renomeado para",
  "for one-click rollback.": "para permitir um recuo imediato.",
  "Import vault": "Importar cofre",
  "Select the exported vault ZIP to restore":
    "Selecionar o ZIP do cofre exportado a restaurar",
  "bloody-level vault ZIP": "ZIP do cofre bloody-level",

  // About, privacy and lifecycle
  "Local-only preferences. Nothing leaves this device.":
    "Preferências apenas locais. Nada sai deste dispositivo.",
  "Local-only clinical lab-PDF tracker with embedded OCR/LLM tiers.":
    "Registo local de PDFs de análises clínicas com níveis OCR/LLM incorporados.",
  "Build profile": "Perfil de compilação",
  Target: "Destino",
  Version: "Versão",
  Frontend: "Frontend",
  "SvelteKit + Tauri WebView (Edge WebView2 on Windows / WKWebView on macOS / WebKitGTK on Linux)":
    "SvelteKit + Tauri WebView (Edge WebView2 no Windows / WKWebView no macOS / WebKitGTK no Linux)",
  License: "Licença",
  "Source-available — see repository LICENSE":
    "Código disponível — consulte o LICENSE do repositório",
  "Privacy & security": "Privacidade e segurança",
  "The vault and managed PDF cache stay on this device. There is no telemetry, no cloud sync, no analytics.":
    "O cofre e a cache de PDFs gerida permanecem neste dispositivo. Não existe telemetria, sincronização na nuvem ou análise de utilização.",
  "full-database encryption with a per-vault key.":
    "encriptação integral da base de dados com uma chave por cofre.",
  "source copies use XChaCha20-Poly1305 with a vault-derived key.":
    "as cópias de origem usam XChaCha20-Poly1305 com uma chave derivada do cofre.",
  "KDF derives the data master key from your password.":
    "a KDF deriva a chave mestre dos dados a partir da sua palavra-passe.",
  "wraps the DMK, with HKDF-SHA-256 sub-derivation.":
    "envolve a DMK, com subderivação HKDF-SHA-256.",
  "support for password-less unlock (PRF extension).":
    "suporte para desbloqueio sem palavra-passe (extensão PRF).",
  "Zero network": "Rede zero",
  "pdfium downloads are build-time only; no runtime egress.":
    "as transferências do pdfium ocorrem apenas durante a compilação; não existe comunicação de saída durante a execução.",
  "Built with": "Construído com",
  "The work of these projects is what makes bloody-level possible. Each is bundled or linked under its own license.":
    "O trabalho destes projetos torna o bloody-level possível. Cada um é incluído ou ligado ao abrigo da sua própria licença.",
  "Each link opens in your default browser via the OS shell — no embedded webview, no redirects through us.":
    "Cada ligação abre no navegador predefinido através da shell do sistema — sem webview incorporada e sem redirecionamentos através de nós.",
  "Open {url}": "Abrir {url}",
  "Author & attribution": "Autor e atribuição",
  Author: "Autor",
  Copyright: "Copyright",
  Disclaimer: "Aviso",
  "This software is for personal record-keeping and trend visualisation only. It is not a medical device. It does not make clinical decisions. Always interpret values with your physician.":
    "Este software destina-se apenas ao registo pessoal e à visualização de tendências. Não é um dispositivo médico. Não toma decisões clínicas. Interprete sempre os valores com o seu médico.",
  "App lifecycle": "Ciclo de vida da aplicação",
  "Refresh the interface, relaunch the native app, or start over with a new empty local instance. Your vault is unchanged by the first two actions.":
    "Atualize a interface, reinicie a aplicação nativa ou comece de novo com uma nova instância local vazia. O seu cofre não é alterado pelas duas primeiras ações.",
  "Reset all preferences": "Repor todas as preferências",
  "Restore appearance, dashboard, charts, compare presets, and tier preferences without deleting vault data.":
    "Restaura a aparência, o painel, os gráficos, as predefinições de comparação e as preferências dos níveis sem eliminar os dados do cofre.",
  "Resetting…": "A repor…",
  "Reset defaults": "Repor predefinições",
  "Restart frontend": "Reiniciar frontend",
  "Reload the current webview and preserve the running native process.":
    "Recarrega a WebView atual e mantém o processo nativo em execução.",
  "Restarting frontend": "A reiniciar frontend",
  "Restart whole app": "Reiniciar aplicação completa",
  "Close and relaunch bloody-level, including its native services.":
    "Fecha e reinicia o bloody-level, incluindo os respetivos serviços nativos.",
  "Restart app": "Reiniciar aplicação",
  "Restarting…": "A reiniciar…",
  "Reset local app": "Repor aplicação local",
  "Permanently delete this local instance and return to the first-run welcome screen.":
    "Elimina permanentemente esta instância local e regressa ao ecrã de boas-vindas inicial.",
  "Reset app": "Repor aplicação",
  "Raw settings": "Definições brutas",
  "Every key in the encrypted settings table. Read-only — power-user inspection only; normal toggles live in the other tabs.":
    "Todas as chaves na tabela de definições encriptada. Só de leitura — destinada apenas a utilizadores avançados; os controlos normais estão nos outros separadores.",

  // Actions, toasts and confirmations
  "Tesseract language data": "Dados de idioma do Tesseract",
  "Delete downloaded Tesseract language data from the app data folder? Native binaries are not affected.":
    "Eliminar os dados de idioma transferidos do Tesseract da pasta de dados da aplicação? Os binários nativos não são afetados.",
  "Delete the managed {label} model files from the app data folder? This cannot be undone.":
    "Eliminar os ficheiros do modelo {label} gerido da pasta de dados da aplicação? Esta ação não pode ser anulada.",
  "Delete {label}": "Eliminar {label}",
  "{label} deleted": "{label} eliminado",
  "The managed files were removed. You can download them again from this page.":
    "Os ficheiros geridos foram removidos. Pode transferi-los novamente a partir desta página.",
  "OS vault enabled": "Cofre do sistema ativado",
  "The vault DMK is now protected by {platform}.":
    "A DMK do cofre está agora protegida por {platform}.",
  "OS vault disabled": "Cofre do sistema desativado",
  "Password and passkey unlock remain available.":
    "O desbloqueio por palavra-passe e chave de acesso continua disponível.",
  "Disable native OS vault unlock? The encrypted database and PDF cache will remain protected, but this device will no longer offer OS-vault unlock. Your vault password is important: keep it strong and recoverable because it protects access to the data on this device.":
    "Desativar o desbloqueio pelo cofre nativo do sistema? A base de dados encriptada e a cache de PDFs continuarão protegidas, mas este dispositivo deixará de oferecer desbloqueio pelo cofre do sistema. A palavra-passe do cofre é importante: mantenha-a forte e recuperável, pois protege o acesso aos dados neste dispositivo.",
  "Disable native OS vault unlock? The encrypted database and PDF cache will remain protected, but this device will no longer offer OS-vault unlock. No vault password is configured, so you will need your registered passkey to unlock. Set a password first if you want password recovery.":
    "Desativar o desbloqueio pelo cofre nativo do sistema? A base de dados encriptada e a cache de PDFs continuarão protegidas, mas este dispositivo deixará de oferecer desbloqueio pelo cofre do sistema. Não está configurada uma palavra-passe do cofre, pelo que precisará da chave de acesso registada para desbloquear. Defina primeiro uma palavra-passe se quiser recuperação por palavra-passe.",
  "The new vault passwords do not match.":
    "As novas palavras-passe do cofre não coincidem.",
  "Vault password changed": "Palavra-passe do cofre alterada",
  "Vault password added": "Palavra-passe do cofre adicionada",
  "Use the new password the next time this vault is locked.":
    "Use a nova palavra-passe da próxima vez que este cofre for bloqueado.",
  "Remove the vault password? You will use your configured passkey or native OS vault to unlock this instance. Keep a recovery method available before continuing.":
    "Remover a palavra-passe do cofre? Usará a chave de acesso configurada ou o cofre nativo do sistema para desbloquear esta instância. Mantenha um método de recuperação disponível antes de continuar.",
  "Remove vault password": "Remover palavra-passe do cofre",
  "Vault password removed": "Palavra-passe do cofre removida",
  "The remaining configured recovery method is still available.":
    "O método de recuperação configurado restante continua disponível.",
  "WebAuthn is not available in this WebView.":
    "WebAuthn não está disponível nesta WebView.",
  "This authenticator did not provide the required PRF capability.":
    "Este autenticador não forneceu a capacidade PRF necessária.",
  "Passkey added": "Chave de acesso adicionada",
  "{label} can now unlock this vault.":
    "{label} pode agora desbloquear este cofre.",
  "this passkey": "esta chave de acesso",
  "Remove {label} from this vault? You will not be able to use it to unlock again.":
    "Remover {label} deste cofre? Não poderá voltar a utilizá-la para desbloquear.",
  "Remove passkey": "Remover chave de acesso",
  "Passkey removed": "Chave de acesso removida",
  "{label} is no longer registered.": "{label} deixou de estar registada.",
  "Rotate master key": "Rodar chave mestre",
  "Rotate the data master key now? The encrypted database and managed PDFs will be re-keyed, and every passkey will need a fresh authenticator assertion. Keep a current backup before continuing.":
    "Rodar agora a chave mestre dos dados? A base de dados encriptada e os PDFs geridos serão re-encriptados e cada chave de acesso precisará de uma nova asserção do autenticador. Mantenha uma cópia de segurança atual antes de continuar.",
  "Master key rotated": "Chave mestre rodada",
  "The encrypted database, managed PDFs, and configured unlock methods remain synchronized.":
    "A base de dados encriptada, os PDFs geridos e os métodos de desbloqueio configurados permanecem sincronizados.",
  "Preset created": "Predefinição criada",
  "“{name}” will appear in Compare's preset bar.":
    "“{name}” aparecerá na barra de predefinições da Comparação.",
  "Preset saved": "Predefinição guardada",
  "“{name}” updated.": "“{name}” atualizada.",
  "Delete preset": "Eliminar predefinição",
  "Delete preset “{name}”? This cannot be undone.":
    "Eliminar a predefinição “{name}”? Esta ação não pode ser anulada.",
  "Preset deleted": "Predefinição eliminada",
  "“{name}” removed.": "“{name}” removida.",
  "Reset preset": "Repor predefinição",
  "Reset “{name}” to its bundled defaults? Your customisations will be lost.":
    "Repor “{name}” nas predefinições incluídas? As suas personalizações serão perdidas.",
  "Preset reset": "Predefinição reposta",
  "“{name}” restored to bundled defaults.":
    "“{name}” restaurada para as predefinições incluídas.",
  "Reset bundled presets": "Repor predefinições incluídas",
  "Restore every bundled preset to its bundled defaults?\n\n• Your edits to bundled presets (Iron panel, Lipids, Hematology, Abnormal, Subclinical, etc.) will be discarded.\n• User-created presets are preserved.":
    "Restaurar todas as predefinições incluídas para os valores predefinidos?\n\n• As suas alterações às predefinições incluídas (Painel de ferro, Lípidos, Hematologia, Anómalos, Subclínicos, etc.) serão eliminadas.\n• As predefinições criadas pelo utilizador serão preservadas.",
  "Bundled presets reset": "Predefinições incluídas repostas",
  'Reload analyte ontology from the bundled seed?\n\n• Re-installs every seed-bundled analyte\'s descriptions, reference ranges, categorical tiers, and aliases — your edits to seed entries will be lost.\n• User-created analytes (source=user) and user-added aliases are preserved.\n• Existing parsed results are not touched. Run "Re-parse all" on Records afterwards if you want stored rows to pick up new ontology fields.':
    'Recarregar a ontologia dos analitos a partir da origem incluída?\n\n• Reinstala as descrições, intervalos de referência, níveis categóricos e aliases de todos os analitos incluídos na origem — as suas alterações às entradas da origem serão perdidas.\n• Os analitos criados pelo utilizador (source=user) e os aliases adicionados pelo utilizador são preservados.\n• Os resultados já analisados não são alterados. Execute "Analisar tudo novamente" em Registos depois se quiser que as linhas guardadas recebam os novos campos da ontologia.',
  "Reload ontology": "Recarregar ontologia",
  "Ontology reloaded": "Ontologia recarregada",
  "{count} analytes installed.": "{count} analitos instalados.",
  "Reloading the current window…": "A recarregar a janela atual…",
  "Restart bloody-level now?\n\nAny unsaved changes in the current view will be discarded. The local vault will remain intact.":
    "Reiniciar o bloody-level agora?\n\nQuaisquer alterações não guardadas na vista atual serão eliminadas. O cofre local permanecerá intacto.",
  "Reset this local bloody-level instance?\n\nThis permanently removes the encrypted vault, password, passkeys, imported reports, PDFs, models, and settings from this device. It cannot be undone. Export a backup first if you may need this data later.":
    "Repor esta instância local do bloody-level?\n\nIsto remove permanentemente deste dispositivo o cofre encriptado, a palavra-passe, as chaves de acesso, os relatórios importados, os PDFs, os modelos e as definições. Não pode ser anulado. Exporte primeiro uma cópia de segurança se puder precisar destes dados mais tarde.",
  "Starting the welcome screen…": "A iniciar o ecrã de boas-vindas…",
  "Reset all bloody-level preferences to their bundled defaults?\n\nThis clears appearance, dashboard, chart, compare, OCR, and other settings. Your vault, reports, PDFs, models, and authentication remain untouched.":
    "Repor todas as preferências do bloody-level para as predefinições incluídas?\n\nIsto limpa as definições de aparência, painel, gráficos, comparação, OCR e outras. O cofre, os relatórios, os PDFs, os modelos e a autenticação permanecem intactos.",
  "Reloading bloody-level with the bundled defaults…":
    "A recarregar o bloody-level com as predefinições incluídas…",
  "Show value labels by default": "Mostrar valores por predefinição",
  "Pinned numeric label at every point. Useful at low cadence; gets noisy with many readings.":
    "Etiqueta numérica fixa em cada ponto. Útil com poucos dados; torna-se confusa com muitas leituras.",
  "Smoothed line by default": "Linha suavizada por predefinição",
  "Bezier-interpolated segments. Off by default — straight segments make stepwise changes obvious.":
    "Segmentos interpolados por Bezier. Desativado por predefinição — os segmentos retos tornam visíveis as mudanças por degraus.",
  "Reference bands by default": "Bandas de referência por predefinição",
  "Coloured horizontal stripe(s) marking the normal / borderline / critical ranges.":
    "Faixas horizontais coloridas que assinalam os intervalos normal, limítrofe e crítico.",
};
