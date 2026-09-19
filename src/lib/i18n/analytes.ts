import { appearance } from "$theme/appearance.svelte";

export type AnalyteTextField =
  "description" | "high_means" | "low_means" | "unit_notes";

/** Human-reviewed locale overlay for the bundled clinical analyte explanations. */
export const analytesPtPT: Record<
  string,
  Partial<Record<AnalyteTextField, string>>
> = {
  hemoglobina: {
    low_means:
      "Anemia. Causas comuns: deficiência de ferro (mais comum), deficiência de vitamina B12/folato, perda de sangue, doença crónica, doença renal, hemólise, supressão da medula óssea.",
    description:
      "Proteína que contém ferro nos glóbulos vermelhos que transporta o oxigénio dos pulmões para os tecidos e o CO₂ de volta. Marcador primário de anemia e capacidade de transporte de oxigénio.",
    high_means:
      "Policitemia, desidratação, tabagismo, hipoxia crónica (altitude elevada, doenças pulmonares/cardíacas) ou perturbações raras da medula óssea (policitemia vera).",
    unit_notes:
      "Referência estratificada por sexo: M 13–17, F 12–15 g/dL. A gravidez reduz os valores normais (limite inferior ~11).",
  },
  eritrocitos: {
    description:
      "Contagem de glóbulos vermelhos – número de glóbulos vermelhos por microlitro de sangue. Interpretado juntamente com a Hb, Hct e os índices (MCV/MCH/MCHC).",
    high_means:
      "Policitemia, desidratação, hipoxia (tabagismo, altitude) ou, em casos raros, policitemia vera.",
    low_means:
      "Anemia (qualquer causa), hemorragia, hemólise, supressão medular, doença renal (baixa EPO).",
  },
  hematocrito: {
    high_means: "O mesmo que Hb elevada – policitemia, desidratação, hipoxia.",
    low_means: "Anemia, hiperhidratação, perda recente de sangue.",
    description:
      "Fração volumétrica de glóbulos vermelhos no sangue total. Rastreia aproximadamente a hemoglobina (Hct ≈ 3 × Hb).",
  },
  vgm: {
    description:
      "Volume Corpuscular Médio – tamanho médio dos glóbulos vermelhos. Crítico para classificar a anemia.",
    low_means:
      "Anemia microcítica (<80 fL): deficiência de ferro (mais comum), talassemia, anemia de doença crónica, envenenamento por chumbo.",
    high_means:
      "Anemia macrocítica (>100 fL): deficiência de B12/folato, abuso de álcool, hipotiroidismo, doença hepática, certos medicamentos (metotrexato, AZT).",
  },
  hgm: {
    high_means:
      "Anemia hipercrómica/macrocítica — deficiência de vitamina B12 ou folato.",
    description:
      "Hemoglobina Corpuscular Média – massa média de hemoglobina por glóbulos vermelhos. Rastreia o MCV de perto.",
    low_means: "Anemia hipocromia – deficiência de ferro ou talassemia.",
  },
  cmhg: {
    description:
      "Concentração Média de Hemoglobina Corpuscular – concentração média de Hb dentro de cada glóbulo vermelho.",
    high_means:
      "Esferocitose hereditária, anemia hemolítica autoimune, desidratação grave. Valores >36 são pouco frequentes e merecem investigação.",
    low_means:
      "Deficiência de ferro, talassemia, envenenamento por chumbo (qualquer coisa que provoque células hipocromáticas).",
  },
  rdw: {
    high_means:
      "Anemias de causa mista, deficiência precoce de ferro/B12/folato, transfusão recente, recuperação pós-hemorragia, mielodisplasia.",
    description:
      "Variação do tamanho dos glóbulos vermelhos. Ajuda a distinguir as causas da anemia (por exemplo, deficiência de ferro = RDW elevado; talassemia = RDW normal).",
    low_means:
      "Raramente clinicamente significativo quando baixo; população de tamanho uniforme de glóbulos vermelhos.",
  },
  leucocitos: {
    description:
      "Contagem total de glóbulos brancos. O diferencial (neutrófilos/linfócitos/etc.) fornece mais informação de diagnóstico do que o total isolado.",
    high_means:
      "Leucocitose — infecção bacteriana, inflamação, stress, corticosteróides, leucemia (valores muito elevados), exercício extenuante recente.",
    low_means:
      "Leucopenia – infeção viral, sépsis grave, doença autoimune, quimioterapia, insuficiência medular, certos medicamentos, deficiência de vitamina B12/folato.",
  },
  neutrofilos: {
    description:
      "Glóbulos brancos mais abundantes; primeiros respondentes à infecção bacteriana. Reportado como percentagem de leucócitos e contagem absoluta.",
    high_means:
      "Infecção bacteriana, inflamação aguda, stress, corticosteróides, tabagismo, reacção leucemóide.",
    low_means:
      "Neutropénia – infeção viral (frequentemente), quimioterapia, destruição autoimune, sépsis grave e avassaladora, insuficiência medular. CAN <500 = risco grave de infeção.",
  },
  eosinofilos: {
    description:
      "Subtipo de leucócitos envolvido em reações alérgicas e infeções parasitárias.",
    low_means:
      "Stress agudo, corticoterapia, doença de Cushing. Geralmente não é clinicamente alarmante quando baixo.",
    high_means:
      "Doença alérgica (asma, eczema, reações medicamentosas), infeções parasitárias, perturbações eosinofílicas, certas leucemias, doença de Addison.",
  },
  basofilos: {
    high_means:
      "Leucemia mieloide crónica (marcador chave), policitemia vera, hipotiroidismo, reações alérgicas, inflamação crónica.",
    description:
      "WBC menos comum. Liberta histamina; envolvido nas respostas alérgicas e inflamatórias.",
    low_means:
      "Raramente clinicamente significativo; infecção aguda, hipertiroidismo, corticosteróides.",
  },
  linfocitos: {
    description:
      "Células imunitárias adaptativas – células T, células B, células NK. Dinamize a resposta de anticorpos e a imunidade mediada por células.",
    high_means:
      "Infeções virais (EBV, CMV, hepatite), tosse convulsa, leucemia linfocítica crónica, linfoma.",
    low_means:
      "Infecção por VIH, imunodepressão, corticosteróides, doenças auto-imunes, stress grave, imunodeficiência congénita.",
  },
  monocitos: {
    description:
      "Maiores leucócitos. Diferencie-se em macrófagos e células dendríticas nos tecidos; importante nas infeções crónicas e inflamações.",
    high_means:
      "Infeção crónica (TB, endocardite), doença inflamatória, doenças autoimunes (lúpus, DII), algumas leucemias (CMML).",
    low_means:
      "Anemia aplástica, leucemia de células pilosas, corticoterapia, infecção aguda grave.",
  },
  plaquetas: {
    description:
      "Fragmentos celulares essenciais para a coagulação do sangue. Forme o tampão primário nos locais da lesão e desencadeie a coagulação.",
    high_means:
      "Trombocitose — reativa (infeção, inflamação, deficiência de ferro, pós-esplenectomia, cancro) ou trombocitemia essencial/doença mieloproliferativa.",
    low_means:
      "Trombocitopenia – PTI, medicamentos (heparina, quinina), DIC, sépsis, insuficiência medular, hiperesplenismo, carências vitamínicas. <50 = risco de hemorragia; <20 = elevado risco de hemorragia espontânea.",
  },
  ferritina: {
    description:
      "Proteína de armazenamento de ferro – o melhor marcador único das reservas totais de ferro no organismo. É também um reagente de fase aguda (aumenta com a inflamação), que pode mascarar a deficiência de ferro.",
    high_means:
      "Sobrecarga de ferro (hemocromatose, transfusões repetidas), inflamação/infecção/cancro (reagente de fase aguda), doença hepática, hipertiroidismo. Muito elevado (>1000) levanta preocupação com hemocromatose ou malignidade.",
    unit_notes:
      "Dependente do sexo e da idade. A inflamação pode normalizar falsamente a ferritina em doentes com deficiência de ferro – em caso de dúvida, combine com a saturação da transferrina.",
    low_means:
      "Deficiência de ferro (teste mais específico). <30 ng/mL sugere deficiência; <15 é praticamente diagnóstico. Pode preceder a anemia em meses.",
  },
  vitamina_b12: {
    description:
      "Vitamina essencial para a síntese de ADN, formação de glóbulos vermelhos e função neurológica. Proveniente de produtos de origem animal. A deficiência provoca anemia megaloblástica e défices neurológicos.",
    high_means:
      "Suplementação excessiva, doença hepática (liberta B12 armazenada), perturbações mieloproliferativas. Raramente é uma preocupação clínica quando elevado.",
    low_means:
      "Anemia perniciosa (perda autoimune de fator intrínseco), dieta vegana estrita, má absorção (doença de Crohn, doença celíaca, pós-cirurgia gástrica), uso de metformina, IBPs. Provoca anemia macrocítica + neuropatia periférica + demência. Trate precocemente – os danos neurológicos podem ser irreversíveis.",
  },
  acido_folico: {
    description:
      "Vitamina B9 essencial para a síntese de ADN e divisão celular. Crítico na gravidez para prevenir defeitos do tubo neural.",
    high_means:
      "Suplementação excessiva. Geralmente não é prejudicial, mas mascara os sintomas neurológicos de deficiência de vitamina B12.",
    low_means:
      "Dieta pobre (poucas folhas verdes), alcoolismo, má absorção, gravidez (necessidade aumentada), metotrexato/fenitoína/sulfassalazina. Causa anemia megaloblástica (indistinguível da deficiência de vitamina B12 no esfregaço) e defeitos do tubo neural na gravidez.",
  },
  d_dimero: {
    description:
      "Produto de degradação da fibrina reticulada. O resultado negativo tem um valor preditivo negativo muito elevado para o TEV em doentes com baixa probabilidade pré-teste – o teste de exclusão de TVP/EP. Muitos falsos positivos, pelo que um valor positivo por si só nunca prevalece.",
    high_means:
      "TEV (TVP/EP), DIC, cirurgia recente/trauma/sepse, malignidade, gravidez (aumenta durante a gestação), idade avançada (utilizar idade limite ajustada por idade × 10 ng/mL após os 50 anos). Útil apenas em conjunto com ferramentas de probabilidade de pré-teste (Wells, Genebra).",
    unit_notes:
      "Referência <500 ng/mL (FEU). Ponto de corte ajustado por idade para >50 anos: idade × 10 ng/mL. Alguns laboratórios reportam DDU (≈ FEU/2) — verifique o ensaio antes de aplicar os pontos de corte.",
    low_means:
      "Abaixo do limite do ensaio. Com poços baixos/intermediários: TEV essencialmente excluído. Não garante qualquer coágulo em doentes com alta probabilidade pré-teste - independentemente da imagem.",
  },
  glicemia: {
    description:
      "Glicemia plasmática em jejum. O principal teste de rastreio para a diabetes mellitus. Reflete o açúcar no sangue atual no momento da colheita.",
    high_means:
      "≥126 mg/dL em jejum em dois dias distintos = diabetes (critérios ADA). 100–125 = glicemia em jejum/pré-diabetes alterada. O stress, as doenças, os esteróides, as refeições recentes e alguns medicamentos também aumentam.",
    unit_notes:
      "Referência ~70–110 mg/dL em jejum. Para converter para mmol/L: divida por 18,018. A HbA1c fornece glicemia média de 2–3 meses.",
    low_means:
      "<70 mg/dL é a hipoglicemia. Causas: sobredosagem de insulina ou sulfonilureia, jejum prolongado, álcool, insuficiência adrenal, doença hepática grave, insulinoma. Sintomas: tremor, sudação, confusão.",
  },
  a1c_ngsp: {
    description:
      "Hemoglobina glicada – a percentagem de moléculas de hemoglobina com glicose ligada. Reflete a glicemia média nos últimos 2–3 meses.",
    high_means:
      "≥6,5% = diabetes (ADA). 5,7–6,4% = pré-diabetes. Valores mais elevados indicam mau controlo glicémico; cada aumento de 1% ≈ 30 mg/dL de glicose média.",
    unit_notes:
      "NGSP %: <5,7 normal · 5,7–6,4 pré-diabetes · ≥6,5 diabetes. Relatórios da IFCC em mmol/mol (escala diferente).",
    low_means:
      "<4% é pouco frequente; sugere perda sanguínea aguda recente, anemia hemolítica ou traço falciforme (falso baixo). A hipoglicemia provocada pelo tratamento excessivo da diabetes também produz níveis baixos de A1c.",
  },
  a1c_ifcc: {
    high_means:
      "≥48 mmol/mol = diabetes. 39–47 = pré-diabetes/risco aumentado. Maior = pior controlo glicémico; cada aumento de 11 mmol/mol ≈ aumento de 1% de NGSP.",
    description:
      "Hemoglobina glicada padronizada pela IFCC em mmol/mol. A mesma biologia que a A1c baseada na % do NGSP — glicemia média durante as 8–12 semanas anteriores — na escala alinhada com o SI utilizada na maior parte da Europa. Conversão: NGSP% = (IFCC × 0,0915) + 2,15.",
    low_means:
      "<20 mmol/mol é pouco frequente. Sugere perda sanguínea aguda recente, anemia hemolítica, traço falciforme (falso baixo) ou diabetes excessivamente tratada com hipoglicemia frequente.",
    unit_notes:
      "<39 normal · 39–47 pré-diabetes · ≥48 diabetes (mmol/mol). Fiável apenas quando a esperança de vida dos eritrócitos é normal – ajuste para hemoglobinopatias, transfusão recente, DRT ou gravidez.",
  },
  glicemia_media_estimada: {
    description:
      "Glicose média estimada derivada da HbA1c através da fórmula ADAG: eAG (mg/dL) = 28,7 × A1c% − 46,7. Traduz uma % de A1c nas unidades diárias de glicose que os doentes vêem nos glicosímetros.",
    low_means:
      "Espelhos baixos A1c. A hipoglicemia frequente na diabetes tratada em excesso é a principal preocupação.",
    unit_notes:
      "Valor derivado — a coluna é informativa. Converta mg/dL → mmol/L dividindo por 18. Tradução útil dirigida ao doente, mas nunca substitui a monitorização domiciliária/tendências de CGM.",
    high_means:
      "Reflete A1c elevado – controlo glicémico deficiente, risco elevado de complicações ao longo do tempo.",
  },
  colesterol_total: {
    high_means:
      "Risco cardiovascular (quando combinado com LDL elevado ou HDL baixo). Hipercolesterolemia familiar, hipotiroidismo, síndrome nefrótico, dieta, obesidade, certos medicamentos.",
    low_means:
      "Hipertiroidismo, desnutrição, má absorção, doença hepática grave, certas anemias. Valores muito baixos (<120) por vezes associados a depressão/risco de suicídio em alguns estudos.",
    description:
      "Colesterol total = LDL + HDL + (TG/5). Um composto de todas as partículas transportadoras de colesterol. As guidelines modernas ponderam mais o LDL e o não-HDL em termos de risco.",
  },
  colesterol_ldl: {
    description:
      "Colesterol de lipoproteína de baixa densidade – a partícula aterogénica primária que deposita colesterol nas paredes arteriais. O principal alvo lipídico para a redução do risco cardiovascular.",
    high_means:
      "Risco de aterosclerose/doença cardiovascular. Hipercolesterolemia familiar, hipotiroidismo, síndrome nefrótico, dieta (gordura saturada, gordura trans).",
    low_means:
      "Hipertiroidismo, doença hepática grave, desnutrição. Valores muito baixos refletem geralmente a terapêutica com estatinas e são tipicamente benéficos.",
    unit_notes:
      "Objectivos de nível de risco: baixo/moderado <115, elevado <100, muito elevado <70 mg/dL. Alguns laboratórios reportam apoB ou não-HDL-C ao lado.",
  },
  colesterol_hdl: {
    description:
      "Colesterol de lipoproteína de alta densidade – transporta inversamente o colesterol dos tecidos periféricos de volta para o fígado. Níveis mais elevados estão associados a menor risco cardiovascular.",
    high_means:
      "Muitas vezes protetor. Genética, exercício aeróbico regular, álcool moderado, certos estrogénios. Muito elevado (>90) sem explicação de estilo de vida está a ser reavaliado como não sendo mais puramente protetor.",
    low_means:
      "Aumento do risco cardiovascular. Estilo de vida sedentário, tabagismo, obesidade, diabetes tipo 2, síndrome metabólica, esteróides anabolizantes, certos medicamentos (betabloqueantes, progestagénios).",
    unit_notes:
      "Referência padrão laboratorial: 35–55 mg/dL (intervalo único). Os pontos de corte clínicos *protetores* estão estratificados por sexo – ≥40 nos homens, ≥50 nas mulheres – mas descrevem limiares de risco CV em vez do intervalo de referência relatado. Abaixo do limite de proteção encontra-se um fator de risco CV independente, independentemente de estar dentro do intervalo laboratorial.",
  },
  trigliceridos: {
    description:
      "Principal forma de gordura no sangue e forma primária de armazenamento de energia. Transportado em VLDL e quilomícrons. Níveis elevados contribuem para o risco cardiovascular e pancreatite.",
    low_means:
      "Hipertiroidismo, desnutrição, má absorção. Raramente clinicamente alarmante quando baixo.",
    unit_notes:
      "Deve ser medido em jejum (≥9–12h). Óptimo <150, limítrofe 150–199, elevado 200–499, muito elevado ≥500. Converta mg/dL → mmol/L em /88,5.",
    high_means:
      "Dieta (açúcar, álcool), resistência à insulina/diabetes tipo 2, obesidade, hipotiroidismo, doenças renais, perturbações genéticas. >500 mg/dL = risco de pancreatite; >1000 = risco elevado de pancreatite.",
  },
  pcr: {
    description:
      "Reagente de fase aguda produzido pelo fígado em resposta à inflamação (em poucas horas). O marcador mais utilizado de inflamação sistémica. PCR de alta sensibilidade (PCR-as) utilizada para a estratificação do risco cardiovascular.",
    high_means:
      "Infeção bacteriana (frequentemente >10), trauma/cirurgia/queimaduras graves, crises autoimunes (AR, LES), doença inflamatória intestinal, cancro, enfarte do miocárdio. PCR-as <1 baixo risco CV, 1-3 médio, >3 elevado.",
    low_means:
      "Sem inflamação. Geralmente uma descoberta tranquilizadora. Os doentes com insuficiência hepática grave não conseguem obter uma resposta de fase aguda.",
    unit_notes:
      "Referência <0,5 mg/dL (ou <5 mg/L). A PCR-us tem a mesma biologia, mas limite de deteção inferior para a estratificação do risco CV.",
  },
  proteinas_totais: {
    description:
      "Soma de todas as proteínas séricas circulantes — albumina (~60%) mais as globulinas. O valor principal de um painel de eletroforese de proteínas séricas (SPE); o fracionamento seguinte informa qual o compartimento alto ou baixo.",
    high_means:
      "Hiperglobulinemia: inflamação/infecção crónica, doença auto-imune, mieloma múltiplo + outras gamopatias monoclonais, doença hepática crónica (hipergamaglobulinemia policlonal). A desidratação produz uma leitura falsamente elevada.",
    low_means:
      "Hipoproteinemia provocada pela albumina: falha na síntese hepática, síndrome nefrótico (perda urinária), enteropatia grave com perda de proteínas, desnutrição, queimaduras, sépsis. Também agamaglobulinemia genética (rara).",
    unit_notes:
      "Referência 6,4–8,3 g/dL. Interprete sempre com as frações SPE e albumina total. Converta g/dL → g/L por ×10.",
  },
  albumina: {
    description:
      "Banda da albumina na eletroforese de proteínas séricas – normalmente reportada como uma percentagem da proteína total e uma concentração absoluta. O maior pico; quantificação densitométrica da proteína transportadora dominante.",
    high_means:
      "Desidratação (relativa). A verdadeira hiperalbuminemia nunca é essencialmente patológica.",
    low_means:
      "Insuficiência de síntese hepática (hepatite crónica, cirrose), síndrome nefrótico (perda urinária), enteropatia perdedora de proteínas, desnutrição, sépsis, terceiro espaçamento. Uma queda aqui empurra as frações relativas de globulina para cima, mesmo quando as suas quantidades absolutas permanecem inalteradas.",
    unit_notes:
      "Referência 55,8–66,1% (eletroforese fracionada). Compare com a albumina sérica absoluta (`Albuminémia`) – as gotas fracionadas podem mascarar o declínio absoluto se a proteína total também for baixa.",
  },
  alfa1_globulinas: {
    high_means:
      "Inflamação/infeção/trauma agudo (aumento em fase aguda de α₁-antitripsina e α₁-glicoproteína ácida), gravidez, terapêutica com estrogénios.",
    low_means:
      "Deficiência de α₁-antitripsina (genética – enfisema + doença hepática de início precoce). Uma banda α₁ plana ou ausente no gel é um achado fundamental de SPE para este diagnóstico.",
    unit_notes:
      "Referência 2,9–4,9% (eletroforese fracionada). Emparelhar com α₁-antitripsina quantitativa se a banda estiver baixa/ausente.",
    description:
      "banda α₁ no SPE. Dominado por α₁-antitripsina com contribuições menores de α₁-glicoproteína ácida, α-fetoproteína, HDL e TBG. Um compartimento reagente de fase aguda.",
  },
  alfa2_globulinas: {
    high_means:
      "Inflamação aguda/crónica (aumento de haptoglobina), síndrome nefrótico (aumento de α₂-macroglobulina – molécula grande retida enquanto a albumina é perdida na urina).",
    low_means:
      "Hemólise intravascular (haptoglobina consumida sem ligação à hemoglobina livre), doença hepática grave, ahaptoglobinemia congénita.",
    description:
      "Banda α₂ no SPE. Principalmente haptoglobina e α₂-macroglobulina. Compartimento reagente de fase aguda.",
    unit_notes:
      "Referência 7,1–11,8% (eletroforese fracionada). O aumento isolado de α₂ + proteinúria intensa é o padrão clássico de SPE da síndrome nefrótica.",
  },
  beta1_globulinas: {
    high_means:
      "Deficiência de ferro (aumento da síntese de transferrina), gravidez/estrogénios.",
    description:
      "Banda β₁ no SPE. Principalmente a transferrina. Alguns laboratórios combinam β₁ e β₂ numa única banda β.",
    low_means:
      "Anemia de doença crónica, doença hepática grave, desnutrição, síndrome nefrótico (perda urinária).",
    unit_notes: "Referência 4,7–7,2% (eletroforese fracionada).",
  },
  beta2_globulinas: {
    high_means: "Inflamação (aumento da fase aguda C3).",
    low_means:
      "Flare ativo de LES ou outra doença que consome complemento (C3 baixo), doença hepática crónica (ponte β-γ desfoca a banda), perda grave de proteína.",
    unit_notes:
      "Referência 3,2–6,5% (eletroforese fracionada). A ponte β-γ no gel é um indício de doença hepática crónica.",
    description:
      "Banda β₂ no SPE. Principalmente complemento C3. Por vezes mostra ponte β-γ na doença hepática crónica.",
  },
  gama_globulinas: {
    description:
      'Banda γ no SPE. Quase na totalidade imunoglobulinas (IgG predominantemente, mais IgA e IgM). A banda SPE mais informativa para o diagnóstico - um "pico monoclonal" nítido (proteína M) nesta região é a pedra angular do rastreio do mieloma múltiplo/MGUS/Waldenström.',
    high_means:
      "Monoclonal: mieloma múltiplo, MGUS, macroglobulinemia de Waldenström, amiloidose primária (pico M – realizar imunofixação). Policlonal: infeção crónica (TB, VIH), doença autoimune (LES, AR), doença hepática crónica (especialmente hepatite autoimune).",
    low_means:
      "Hipogamaglobulinemia: IDCV, agamaglobulinemia ligada ao X, perda grave de proteínas, terapêutica imunossupressora, mieloma múltiplo (supressão de classes de Ig não tumorais).",
    unit_notes:
      "Referência 11,1–18,8% (eletroforese fracionada). Qualquer pico estreito aqui justifica a eletroforese de imunofixação (IFE) mais cadeias leves livres de soro para o caracterizar.",
  },
  ag_ratio: {
    description:
      "Albumina / (Proteína Total − Albumina). Um descritor resumido do padrão SPE. A relação invertida (<1) significa geralmente um ganho relativo de globulinas ou uma perda relativa de albumina.",
    high_means:
      "A/G elevada reflete, geralmente, deficiência de globulina (imunodeficiência, desnutrição grave).",
    low_means:
      "Relação invertida (<1) — doença hepática crónica, mieloma múltiplo/gamopatia monoclonal, inflamação crónica, síndrome nefrótico (perda seletiva de albumina).",
    unit_notes:
      "Referência 1.2–2.2. Valor derivado — significativo apenas com o SPE subjacente.",
  },
  albuminemia: {
    low_means:
      "Insuficiência da síntese hepática (cirrose, hepatite grave), síndrome nefrótico (perda urinária), enteropatia perdedora de proteínas, má absorção, queimaduras graves, sépsis (supressão de fase aguda), inflamação crónica. Promove o edema ao diminuir a pressão oncótica quando <2,5 g/dL.",
    high_means:
      "Desidratação. A verdadeira hiperalbuminemia nunca é essencialmente patológica.",
    description:
      "Concentração absoluta de albumina sérica medida colorimetricamente (roxo de bromocresol, BCP – mais específico do que os métodos mais antigos de verde de bromocresol). Reflete a síntese hepática, a perda de proteínas e o estado nutricional geral. A longa semivida (~20 dias) torna-o um marcador de mudança lenta – e não um rastreador sensível de insultos agudos.",
    unit_notes:
      "Referência 3,5–5,0 g/dL. Utilize preferencialmente métodos baseados em BCP (o BCG mais antigo sobrestima em 0,2–0,5 g/dL). Converter mg/dL → µmol/L: ×151. Corrija sempre o cálcio sérico para albumina (+0,8 mg/dL Ca por 1 g/dL de albumina abaixo de 4,0).",
  },
  uricemia: {
    description:
      "Produto final do metabolismo das purinas. O ácido úrico elevado provoca gota e pedras nos rins; também independentemente associada à hipertensão e ao risco cardiovascular.",
    high_means:
      "Hiperuricemia: excesso alimentar (carne, álcool, frutose), doença renal, diuréticos (especialmente tiazídicos), síndrome de lise tumoral, psoríase, toxicidade por chumbo. Gota sintomática normalmente >6,8 mg/dL.",
    low_means:
      "SIADH, gravidez (leve), doença de Wilson, síndrome de Fanconi, doença hepática grave, medicamentos uricosúricos (probenecida, losartan).",
    unit_notes:
      "M 3,5–7,2, F 2,6–6,0 mg/dL. Meta no tratamento da gota <6 (frequentemente <5 se for tofácea). Converter para µmol/L: ×59,48.",
  },
  uremia: {
    description:
      "Produto residual azotado do metabolismo das proteínas, filtrado pelos rins. Menos específica que a creatinina para a função renal – varia de acordo com a ingestão de proteínas, hidratação e hemorragia.",
    high_means:
      "Disfunção renal (combinada com creatinina elevada), desidratação, dieta rica em proteínas, hemorragia gastrointestinal (carga proteica), insuficiência cardíaca, obstrução urinária, estados catabólicos. A relação BUN/creatinina >20 sugere causa pré-renal.",
    low_means:
      "Baixa ingestão de proteínas, doença hepática grave (ureia sintetizada no fígado), SIADH, hiperhidratação, gravidez tardia.",
    unit_notes:
      "Referência 10–50 mg/dL ureia (os laboratórios portugueses reportam ureia, e não BUN). Para converter ureia mg/dL ↔ BUN mg/dL: BUN ≈ ureia / 2,14. Combinar com creatinina para avaliação renal.",
  },
  creatininemia: {
    high_means:
      "Redução da função renal (lesão renal aguda ou doença renal crónica), desidratação, grande massa muscular, lesão muscular (rabdomiólise), dieta rica em proteínas, certos medicamentos (cimetidina, trimetoprim).",
    low_means:
      "Massa muscular reduzida (idosos, desnutrição, doenças de perda muscular), gravidez (filtração aumentada), doença hepática.",
    unit_notes:
      "Dependente da massa muscular – o mesmo valor significa pior função renal numa pessoa pequena e frágil do que num atleta grande. Interprete sempre junto com a TFGe.",
    description:
      "Produto residual do metabolismo muscular (degradação da creatina), filtrado pelos rins. O marcador mais comum da função renal, mas fica atrás da TFGe para a deteção precoce de doenças.",
  },
  tfge_ckd_epi: {
    high_means:
      "Geralmente bom – elevada capacidade de filtração. A hiperfiltração (diabetes precoce, gravidez, dieta rica em proteínas) pode produzir valores que parecem tranquilizadores, mas mascaram o desenvolvimento da patologia.",
    description:
      "Taxa estimada de filtração glomerular — calculada a partir da creatinina, idade e sexo (equação CKD-EPI 2009). O indicador primário moderno da função renal e do estadiamento da DRC.",
    low_means:
      "Função renal reduzida. Estágios da DRC: G1 ≥90 (normal/alto), G2 60–89 (leve), G3a 45–59 (leve-moderado), G3b 30–44 (moderado-grave), G4 15–29 (grave), G5 <15 (insuficiência renal).",
    unit_notes:
      "Referência: ≥60 mL/min/1,73 m². Persistentemente <60 durante ≥3 meses = DRC por definição. Emparelhe com a relação albumina/creatinina na urina (RAC) para estadiar completamente.",
  },
  ast: {
    description:
      "A enzima hepática está também presente no coração, nos músculos e nos glóbulos vermelhos. Menos específico do fígado do que o ALT – também pode aumentar devido a danos musculares.",
    high_means:
      "Lesão hepatocelular (viral/alcoólica/DHGNA/hepatite induzida por fármacos), lesão muscular (rabdomiólise, exercício intenso), IM (marcador mais antigo, agora substituído por troponina), hemólise.",
    low_means:
      "Raramente clinicamente significativo. Deficiência de vitamina B6 ou doença hepática em fase terminal (não resta tecido hepático).",
    unit_notes:
      "A relação AST/ALT >2 com níveis elevados sugere doença hepática alcoólica; relação <1 sugere viral/DHGNA. Sempre avaliados em conjunto.",
  },
  alt: {
    description:
      "A aminotransferase mais específica do fígado. Aumenta com a lesão hepatocelular – o marcador canónico de lesão hepática aguda ou crónica.",
    high_means:
      "Hepatite viral, hepatite alcoólica, DHGNA/EHNA, lesão hepática induzida por fármacos (paracetamol, estatinas, antibióticos), hepatite autoimune, hepatite isquémica. >1000 U/L sugere lesão maciça aguda.",
    low_means:
      "Raramente clinicamente significativo. Pode refletir deficiência de vitamina B6 ou perda extensa de tecido hepático.",
  },
  ggt: {
    description:
      "Enzima que se encontra no fígado e no trato biliar. Marcador sensível para o uso de álcool e obstrução biliar; ajuda a confirmar a origem hepática da fosfatase alcalina elevada.",
    high_means:
      "Uso de álcool (marcador sensível), colestase/obstrução biliar, esteatose hepática, indução enzimática induzida por fármacos (fenitoína, barbitúricos), doença pancreática.",
    unit_notes:
      "Se tanto a ALP como a GGT estiverem elevadas → fonte biliar/colestática. Se a ALP estiver elevada, mas a GGT estiver normal → provável fonte óssea.",
    low_means:
      "Hipotiroidismo. Raramente clinicamente significativo quando baixo.",
  },
  fosfatase_alcalina: {
    description:
      "Enzima abundante no fígado (ductos biliares), ossos (osteoblastos), placenta e intestino. Utilizado para detetar doença hepática colestática e remodelação óssea. Fonte de elevação determinada pela GGT.",
    high_means:
      "Colestase/obstrução biliar (emparelhada com GGT elevada), doença óssea (Paget, osteomalácia, fraturas, crianças em crescimento — emparelhada com GGT NORMAL), gravidez (ALP placentária), infiltração hepática, hipertiroidismo.",
    low_means:
      "Hipotiroidismo, desnutrição, deficiência de vitamina C/zinco/magnésio, hipofosfatasia (genética rara), transfusão recente.",
    unit_notes:
      "Referência 40–130 U/L (varia consoante o laboratório e a idade). Normalmente elevado em crianças em crescimento e durante a gravidez.",
  },
  bilirrubinemia_total: {
    description:
      "Produto de decomposição do heme. Bilirrubina total = não conjugada (indireta, proveniente da degradação dos glóbulos vermelhos) + conjugada (direta, processada pelo fígado). Causa icterícia quando elevado acima de ~2-3 mg/dL.",
    low_means:
      "Geralmente não é clinicamente significativo. Pode refletir baixa rotatividade do heme.",
    unit_notes:
      "Referência 0,2–1,2 mg/dL. Solicitar bilirrubina direta para fracionar quando o total é elevado. Converter mg/dL → µmol/L: ×17,1.",
    high_means:
      "Pré-hepática (hemólise — predomina a indirecta), hepática (hepatite, cirrose — ambas aumentam), pós-hepática (colestase/obstrução — predomina a directa). Síndrome de Gilbert: hiperbilirrubinemia indireta ligeira e isolada, benigna.",
  },
  ldh: {
    description:
      "Enzima ubíqua libertada de qualquer tecido danificado. Marcador inespecífico de lesão celular. Utilizado em oncologia (carga tumoral/resposta ao tratamento) e para despiste de hemólise.",
    high_means:
      "Hemólise (frequentemente muito elevada), lesão tecidular de qualquer tipo: IM (marcador mais antigo), lesão hepática, lesão pulmonar/EP, lesão muscular, anemia hemolítica, linfoma/leucemia/cancro metastático (marcador de resposta ao tratamento).",
    low_means:
      "Raramente clinicamente significativo. Algumas deficiências genéticas da subunidade LDH.",
  },
  ck_total: {
    high_means:
      "Rabdomiólise (frequentemente> 5.000-10.000), exercício extenuante, injeções intramusculares, trauma, miopatia por estatinas, distrofia muscular, hipotiroidismo, IM (CK-MB).",
    description:
      "Enzima abundante no músculo esquelético, coração e cérebro. Liberado com lesão muscular. A CK total aumenta em qualquer lesão muscular; A isoforma CK-MB é específica do coração (amplamente substituída pela troponina).",
    low_means:
      "Massa muscular reduzida, repouso prolongado, metotrexato/esteróides (crónico). Raramente uma bandeira.",
  },
  calcemia: {
    description:
      "Cálcio sérico total – cerca de 50% ligado à albumina, 40% ionizado (ativo), 10% complexado. Crítico para a função muscular/nervosa, coagulação e ossos. Interprete sempre com albumina (correcção: +0,8 mg/dL por 1 g/dL de albumina inferior a 4,0).",
    high_means:
      "Hiperparatiroidismo (causa ambulatória mais comum), malignidade (causa mais comum em doentes internados), toxicidade por vitamina D, sarcoidose, leite alcalino, diuréticos tiazídicos, imobilização, lítio.",
    low_means:
      "Hipoparatiroidismo, deficiência de vitamina D, doença renal, deficiência de magnésio, pancreatite, sépsis, citrato transfusional, albumina baixa (baixa aparente – cálculo corrigido).",
    unit_notes:
      "Referência 8,7–10,4 mg/dL. Gravemente anormal provoca tetania (baixa) ou arritmia/IRA (alta). Converter para mmol/L: ÷4,008.",
  },
  magnesiemia: {
    description:
      "Cátion essencial para o ATP, co-factor enzimático (>300 enzimas), função muscular/nervosa, osso. Principalmente intracelular – os níveis séricos podem subestimar o défice corporal total.",
    low_means:
      "Diuréticos (alça, tiazida), IBP, alcoolismo, má absorção, síndrome de realimentação, cetoacidose diabética. Causa hipocaliémia refratária e hipocalcémia. Sintomas: tetania, arritmia.",
    high_means:
      "Insuficiência renal, ingestão excessiva (laxantes, antiácidos), lise tumoral, hemólise grave, insuficiência supra-renal. Sintomas em >4-5: fraqueza, hiporreflexia, arritmia.",
  },
  fosfatemia: {
    description:
      "Principalmente armazenado nos ossos com cálcio. Crítico para ATP, ADN, membranas celulares. Inversamente relacionado com o cálcio via regulação do PTH e da vitamina D.",
    high_means:
      "Insuficiência renal (mais comum), hipoparatiroidismo, síndrome de lise tumoral, rabdomiólise, toxicidade por vitamina D.",
    low_means:
      "Síndrome de realimentação (com risco de vida), hiperparatiroidismo, deficiência de vitamina D, alcoolismo, recuperação de cetoacidose diabética, antiácidos.",
  },
  vitamin_d_25oh: {
    description:
      "Forma de armazenamento de vitamina D – o teste padrão para o estado de vitamina D. Produzido na pele a partir da luz solar e hidroxilado no fígado. Essencial para a homeostasia do cálcio/osso e função imunitária.",
    high_means:
      "Suplementação excessiva (>100 ng/mL = risco de toxicidade: hipercalcemia, cálculos renais, calcificação dos tecidos moles). O estilo de vida rico em sol raramente produz toxicidade.",
    low_means:
      "Exposição solar insuficiente, pele escura, obesidade, má absorção (celíaca, DII, bypass gástrico), doença renal/hepática. Ligado a doenças ósseas (osteomalácia, raquitismo), aumento do risco de fraturas, possíveis efeitos imunológicos.",
    unit_notes:
      "Níveis categóricos (ng/mL): <10 deficiente · 10–30 insuficiente · 30–100 suficiente · >100 toxicidade. Converta para nmol/L por ×2,5.",
  },
  natremia: {
    description:
      "Principal catão extracelular. Reflete mais o equilíbrio hídrico do que o equilíbrio de sódio – regulado principalmente pelo ADH. As alterações agudas provocam sintomas neurológicos (edema cerebral ou desmielinização osmótica).",
    high_means:
      "Hipernatrémia: défice de água livre (ingestão insuficiente, diabetes insípida, diurese), perdas gastrointestinais, sudação profusa, intoxicação por sal. Risco de encolhimento celular/hemorragia cerebral.",
    low_means:
      "Hiponatremia: SIADH (cancro, doença pulmonar, medicamentos), insuficiência cardíaca/fígada/rim, diuréticos, polidipsia primária, hipotiroidismo, doença de Addison. Risco de edema cerebral, convulsões.",
  },
  kaliemia: {
    description:
      "Principal catão intracelular. Crítico para a função cardíaca e neuromuscular. Fortemente regulamentado; pequenas alterações séricas podem causar arritmias.",
    high_means:
      "Hipercaliémia: insuficiência renal, IECA/BRA/espironolactona, degradação dos tecidos (rabdo, lise tumoral), acidose, doença de Addison, transfusão de sangue. Risco: pico de ondas T, arritmia fatal (>6,5 = emergência).",
    low_means:
      "Hipocaliémia: diuréticos (alça, tiazida), perdas gastrointestinais (vómitos, diarreia), deficiência de magnésio, alcalose, realimentação. Risco: ondas U, arritmia, fraqueza.",
  },
  cloremia: {
    description:
      "Principal anión extracelular. Rastreia o sódio de perto; a diferença (juntamente com o bicarbonato) determina o anion gap, que classifica a acidose metabólica.",
    high_means:
      "Acidose metabólica hiperclorémica (acidose tubular renal, diarreia, sobrecarga salina normal), desidratação, hiperventilação.",
    low_means:
      "Perdas gastrointestinais com H+ (vómitos, sucção de NG), compensação da acidose respiratória, diuréticos, SIADH.",
  },
  tsh: {
    description:
      "Hormona hipofisária que sinaliza à tiroide para produzir T4/T3. O teste único mais sensível da função tiroideia – anormal antes da alteração de T4/T3 no início da doença.",
    high_means:
      "Hipotiroidismo primário (doença de Hashimoto, pós-radiação, pós-cirúrgico, deficiência de iodo, certos medicamentos). O tumor hipofisário secretor de TSH é raro. A recuperação de doenças não tiroideias pode aumentar transitoriamente.",
    unit_notes:
      "Referência 0,35–5,5 mUI/L (dependente do ensaio). A gravidez necessita de pontos de corte específicos para o trimestre (limite inferior superior). Hipotiroidismo subclínico: TSH elevado, T4L normal.",
    low_means:
      "Hipertiroidismo primário (graves, nódulo tóxico, tiroidite), reposição excessiva de hormona tiroideia, hipotiroidismo secundário (insuficiência hipofisária – T4 também baixa), gravidez no primeiro trimestre.",
  },
  ft4: {
    description:
      "Tiroxina livre (não ligada, biologicamente ativa). Medido juntamente com o TSH para confirmar e estadiar a disfunção tiroideia.",
    high_means:
      "Hipertiroidismo (graves, nódulo tóxico), reposição excessiva de T4, transitório com tiroidite, certos medicamentos.",
    low_means:
      "Hipotiroidismo evidente (quando também TSH elevado), hipotiroidismo secundário (causa hipofisária – TSH inadequadamente normal/baixo), doença não tiroideia grave.",
  },
  prolactina: {
    description:
      "Hormona hipofisária que estimula a produção de leite (lactação) e influencia a função reprodutiva. Elevado fora da gravidez/lactação sugere distúrbio hipofisário ou hipotalâmico.",
    low_means:
      "Síndrome de Sheehan (enfarte hipofisário pós-parto), tumores hipofisários que comprimem as células produtoras de prolactina, agonistas da dopamina. Geralmente clinicamente insignificante quando baixo.",
    high_means:
      "Gravidez, lactação, prolactinoma (adenoma hipofisário), hipotiroidismo (TRH estimula a prolactina), medicamentos (antipsicóticos, metoclopramida, opióides, estrogénios), stress, estimulação da parede torácica, doença renal.",
    unit_notes:
      "Referência: M 4,0–15,2, F 4,8–23,3 ng/mL (não grávida). Gravidez 10–250. A macroprolactinemia (grande forma biologicamente inativa) pode provocar falsas elevações.",
  },
  fsh: {
    description:
      "Hormona hipofisária que impulsiona a gametogénese. Nas mulheres, recruta folículos ováricos; nos homens, suporta a espermatogénese através das células de Sertoli. Fortemente dependente do ciclo nas mulheres e aumenta drasticamente após a menopausa.",
    high_means:
      "Mulher: pós-menopausa (esperada, frequentemente >25), insuficiência ovárica prematura, disgenesia gonadal (Turner). Masculino: insuficiência testicular primária (Klinefelter, pós-orquite, quimioterapia). Adenoma hipofisário secretor de FSH (raro).",
    low_means:
      "Insuficiência hipotalâmica/hipófise (Kallmann, pan-hipopituitarismo, traumatismo cranioencefálico, tumores), anorexia/exercício extremo (amenorreia hipotalâmica funcional), hiperprolactinemia, terapêutica com agonistas da GnRH.",
    unit_notes:
      "Masculino 1,4–18,1; a referência feminina depende muito da fase do ciclo. Pós-menopausa >25,8 mUI/mL. Utilizado em conjunto com LH (relação LH:FSH >2 na SOP) e estradiol/testosterona.",
  },
  lh: {
    description:
      "Hormona hipofisária que desencadeia a ovulação nas mulheres e estimula as células de Leydig a produzir testosterona nos homens. A relação LH:FSH é útil no diagnóstico (>2 na SOP).",
    low_means:
      "Insuficiência hipotalâmica/hipofisária, anorexia, hiperprolactinemia, distúrbios alimentares, supressão de GnRH, uso de androgénios exógenos.",
    high_means:
      "Mulher: pico ovulatório (normal, transitório), pós-menopausa, insuficiência ovárica prematura, SOP (alteração da proporção). Masculino: insuficiência testicular primária. Adenoma secretor de LH (raro).",
  },
  estradiol: {
    description:
      "17-beta-estradiol – o estrogénio endógeno mais potente. Nas mulheres, os níveis oscilam drasticamente ao longo do ciclo menstrual (mais baixos no início do ciclo folicular, com picos na ovulação). Nos homens, os níveis são baixos e estáveis ​​(aromatizados a partir da testosterona).",
    low_means:
      "Feminino: pós-menopausa, insuficiência ovárica, amenorreia hipotalâmica, anorexia, exercício intenso, contracetivos orais. Masculino: raramente clinicamente significativo quando baixo.",
    high_means:
      "Feminino: pico de ovulação (normal), gravidez, tumores do ovário, hipertiroidismo. Masculino: tumores secretores de estrogénios, ginecomastia, doença hepática grave.",
    unit_notes:
      "A interpretação feminina exige conhecer a fase do ciclo. Referência masculina ~7-43 pg/mL. Para converter pg/mL → pmol/L multiplique por 3,671.",
  },
  testosterona_total: {
    description:
      "Testosterona circulante total – a soma da testosterona livre (~2%), ligada à albumina (~38%) e ligada ao SHBG (~60%). O teste de rastreio padrão para o hipogonadismo nos homens e o excesso de androgénios nas mulheres. Melhor desenhado de manhã.",
    high_means:
      "Masculino: uso de esteróides anabolizantes, reposição de testosterona, tumor adrenal/testicular, hiperplasia adrenal congénita. Mulher: SOP (mais comum), hiperplasia adrenal congénita, tumor ovárico/adrenal, androgénios exógenos.",
    low_means:
      "Masculino: hipogonadismo primário ou secundário (Klinefelter, doença hipofisária, uso de opióides, obesidade, doença crónica, envelhecimento). Feminino: raramente diagnóstico – insuficiência adrenal, hipopituitarismo, uso de contracetivos orais.",
    unit_notes:
      "M 241–827 ng/dL (diminui com a idade – debate sobre o hipogonadismo de início tardio). F 14–76 ng/dL. Total falsamente reduzido por SHBG elevado (estrogénio oral, hipertiroidismo, doença hepática) – medir a testosterona livre ou SHBG para distinguir.",
  },
  testosterona_livre: {
    high_means:
      "O mesmo que testosterona total elevada – esteróides anabolizantes, T exógena, SOP na mulher, tumores adrenais/gonadais. O T livre também aumenta quando o SHBG está baixo (obesidade, diabetes tipo 2, hipotiroidismo).",
    description:
      "A fração biologicamente ativa da testosterona (não ligada ao SHBG ou à albumina). Útil quando o SHBG é anormal (o SHBG elevado mascara a testosterona total baixa – o quadro torna-se mais claro com T livre).",
    low_means:
      "Hipogonadismo (com total baixo). T livre baixo isolado com total normal sugere SHBG elevado (hipertiroidismo, estrogénio oral, idade avançada, cirrose hepática).",
    unit_notes:
      "Adulto M (20-50 anos) 8,69–54,69 pg/mL; >50y ligeiramente inferior. Adulto F 0,29–3,18 pg/mL. A medição direta é preferível ao T livre calculado do total + SHBG.",
  },
  shbg: {
    description:
      "Glicoproteína produzida pelo fígado que se liga às hormonas sexuais (testosterona > estradiol), regulando a sua fracção biodisponível. Indispensável para interpretar a testosterona total ou estradiol.",
    high_means:
      "Hipertiroidismo, cirrose hepática, estrogénios orais (ACOs), TSH, anorexia, antiepiléticos, envelhecimento no homem. SHBG elevado → menos hormona livre → pode mascarar o hipogonadismo com T total normal.",
    low_means:
      "Hipotiroidismo, obesidade/síndrome metabólica/diabetes tipo 2, SOP, androgénios/esteróides exógenos, excesso de hormona de crescimento, síndrome de Cushing. SHBG baixo → mais hormona livre → aumento dos cálculos de T livre.",
    unit_notes:
      "M 10–57, F 18–144 nmol/L (varia de acordo com o laboratório). A testosterona livre calculada utiliza T + SHBG total ± albumina.",
  },
  dht: {
    description:
      "Androgénio mais potente que a testosterona, produzido a partir da T pela 5-alfa-redutase na pele, próstata, folículos pilosos e fígado. Impulsiona o desenvolvimento sexual masculino externo, o crescimento da próstata, os pelos do corpo e a calvície masculina no couro cabeludo.",
    high_means:
      "Regulação positiva da 5-alfa-redutase, esteróides anabolizantes, aumento da próstata, hirsutismo nas mulheres, alopécia androgenética.",
    low_means:
      "Deficiência de 5-alfa-redutase (DSD), terapêutica com finasterida/dutasterida (intencional). O baixo DHT em relação ao T explica o fenótipo genital no 5-ARD.",
  },
  acth: {
    description:
      "Hormona hipofisária que estimula o córtex da supra-renal a produzir cortisol. Diurno: picos entre as 6h e as 8h, mínimos à meia-noite. Melhor desenhado às 8h com cortisol emparelhado.",
    high_means:
      "Insuficiência adrenal primária (Addison), adenoma hipofisário secretor de ACTH (doença de Cushing), ACTH ectópico (cancro do pulmão de pequenas células, carcinoide), hiperplasia adrenal congénita.",
    low_means:
      "Cushing adrenal (cortisol autónomo suprime ACTH), corticoterapia exógena, insuficiência adrenal secundária/terciária (insuficiência hipofisária ou hipotalâmica).",
    unit_notes:
      "Referência 7,2–63,3 pg/mL às 8h. Altamente sensível ao stress; questões de manuseamento de amostras (arrefecidas, EDTA, processamento rápido).",
  },
  cortisol_basal: {
    description:
      "Glicocorticóide primário – regula a glicose, a pressão arterial, a resposta imunitária e a resposta ao stress. Altamente diurno: pico matinal (~6-8h), nadir noturno.",
    high_means:
      "Síndrome de Cushing (qualquer causa), stress/doença aguda, corticosteróides exógenos, depressão grave, alcoolismo. Utilize o teste de supressão de dexametasona de 1 mg durante a noite ou cortisol livre de urina de 24 horas para confirmar.",
    low_means:
      "Insuficiência adrenal (Addison, secundária, terciária), crise adrenal, doença hipofisária, retirada abrupta de esteróides, hiperplasia adrenal congénita (algumas formas).",
    unit_notes:
      "Manhã (8h) referência 6,2–19,4 µg/dL; noite normalmente <50% da manhã. Cortisol aleatório de valor limitado devido à oscilação diurna.",
  },
  igf1: {
    high_means:
      "Acromegalia/gigantismo (adenoma secretor de GH hipofisário), terapêutica com GH exógeno, gravidez, puberdade.",
    low_means:
      "Deficiência de GH (insuficiência hipofisária), desnutrição grave, insuficiência hepática, diabetes tipo 1 (mal controlada), idade avançada (diminui naturalmente).",
    unit_notes:
      "Fortemente dependente da idade: atinge o pico da puberdade, diminui com a idade. Adulto varia entre 109 a 284 ng/mL (varia consoante o laboratório e a década de idade).",
    description:
      "Mediador da ação da hormona do crescimento (GH) produzida pelo fígado. Estável ao longo do dia (ao contrário do GH pulsátil), tornando-se o exame padrão para a acromegalia e deficiência de GH.",
  },
  troponina_i: {
    high_means:
      "IM agudo (STEMI / NSTEMI), miocardite, descompensação de insuficiência cardíaca aguda, EP com distensão do coração direito, sépsis grave, trauma cardíaco, exercício intenso (raro, de baixo nível), cirurgia cardíaca.",
    low_means:
      "Abaixo do limite de deteção — nenhuma lesão cardíaca aguda detetada. Interprete sempre com o tempo e medições seriais.",
    description:
      "Proteína contrátil específica do coração libertada para o sangue quando o miocárdio é danificado. Biomarcador padrão para enfarte do miocárdio (substituído CK-MB).",
  },
  troponina_i_hs: {
    description:
      "Ensaio de troponina de alta sensibilidade – mesma biologia da troponina padrão, mas deteta concentrações muito mais baixas, permitindo algoritmos rápidos de regra de entrada/exclusão de IM de 0/1h ou 0/2h no rastreio da dor torácica.",
    high_means:
      "IM agudo (mais comum). Além disso: miocardite, takotsubo, insuficiência cardíaca aguda, EP, sépsis, insuficiência renal (elevação crónica ligeira), exercício de resistência. A trajetória é mais importante do que um valor único.",
    low_means:
      "Resultado negativo (abaixo do ponto de corte do percentil 99). Combinado com um score clínico de baixo risco, valor preditivo negativo muito elevado para IM em 1-2h.",
    unit_notes:
      "Pontos de corte do percentil 99 específicos do sexo: M ≤19, F ≤11 ng/L (dependente do fornecedor). O delta entre sorteios seriados é diagnóstico – um padrão de subida/descida com pelo menos um valor acima de 99 = lesão aguda.",
  },
  ige_total: {
    description:
      "Imunoglobulina E Total — classe de anticorpos envolvidos nas reações alérgicas e infeções parasitárias. O normal para os adultos é geralmente <120 UI/mL, mas os intervalos de referência são fortemente estratificados por idade (crianças muito mais baixas).",
    high_means:
      "Doença alérgica (asma, dermatite atópica, rinite alérgica), infeções parasitárias, síndromes de hiper-IgE, certas imunodeficiências, aspergilose broncopulmonar alérgica. Muito elevado (>1000) levanta preocupação para estas condições específicas.",
    unit_notes:
      "Referência adulto <120 UI/mL. A referenciação pediátrica aumenta com a idade (recém-nascido ~<1, 7 anos ~<160, 10 anos ~<570). Para converter: 1 UI/mL = 2,4 ng/mL.",
    low_means:
      "Imunodeficiência comum variável, hipogamaglobulinemia, ataxia-telangiectasia. Muitas vezes clinicamente insignificante quando isolado.",
  },
  ige_inalantes_multiplo: {
    description:
      "Teste de rastreio para sensibilização mediada por IgE a alergénios comuns transportados pelo ar (ácaros, pêlos de animais, baratas, esporos de fungos, pólenes). Retorna Positivo/Negativo. A positividade NÃO confirma alergia clínica – requer correlação dos sintomas.",
    high_means:
      "Positivo: Anticorpos IgE presentes contra ≥1 dos alergénios do painel. Sugere predisposição atópica; emparelhar com a história clínica e considerar testes resolvidos por componentes para identificar o alergénio específico.",
    low_means:
      "Negativo: sem sensibilização detetável a alergénios inalantes. Exclui eficazmente a alergia a inalantes mediada por IgE como causa do sintoma.",
    unit_notes:
      "Qualitativo — sem intervalo de referência numérica. Bandeira Positivo é informativa; correlação clínica necessária.",
  },
  vsg: {
    description:
      "Taxa de sedimentação dos glóbulos vermelhos em 1 hora – um marcador inespecífico de inflamação. Barato, lento a mudar, mantém relevância para o diagnóstico de arterite temporal (frequentemente >50) e rastreio da atividade da RA/PMR.",
    high_means:
      "Inflamação (qualquer causa), infeção, malignidade (especialmente mieloma – pode ser muito elevada), gravidez, anemia, insuficiência renal, idade avançada. Muito elevado (>100) levanta preocupação para mieloma, arterite temporal, cancro oculto.",
    low_means:
      "Policitemia, anemia falciforme, insuficiência cardíaca grave, hipofibrinogenemia, microcitose. Raramente a pista clínica primária.",
    unit_notes:
      "Dependente do sexo e da idade. Referência Westergren m <15, f <20 mm/h <50y; os doentes mais velhos podem ter ~idade/2. A PCR sobe e desce mais rapidamente – emparelhe-os.",
  },
  reticulocitos: {
    description:
      "Glóbulos vermelhos jovens e imaturos – uma medida da resposta eritropoiética da medula. Calcule a contagem corrigida de reticulócitos ou o índice de produção de reticulócitos na anemia para distinguir a subprodução da perda de sangue/hemólise.",
    high_means:
      "Resposta rápida da medula: hemólise, perda aguda de sangue, reposição de ferro/B12/folato, terapêutica com EPO.",
    low_means:
      "Subprodução de medula: deficiência de ferro/B12/folato (não tratada), anemia aplástica, doença renal (baixa EPO), infiltração medular, quimioterapia.",
    unit_notes:
      "Referência 0,5–2,5%. Contagem absoluta mais útil na anemia: <50.000/µL = resposta inadequada, >100.000/µL = adequada.",
  },
  tp_inr: {
    description:
      "Via de coagulação extrínseca + comum. O INR (International Normalized Ratio) uniformiza o PT entre laboratórios/reagentes. Padrão para monitorização de varfarina e rastreio para deficiência de fator sintetizado pelo fígado.",
    high_means:
      "Terapêutica com varfarina (alvo 2–3 na maioria das indicações, 2,5–3,5 válvulas mecânicas), doença hepática, deficiência de vitamina K, DIC, deficiência de fator II/VII/IX/X, anticoagulante lúpico. INR >5 = risco de hemorragia.",
    unit_notes:
      "Referência INR 0,8–1,2 (não tratado). Varfarina terapêutica: 2,0–3,0 (maioria), 2,5–3,5 (válvula aórtica/mitral mecânica).",
    low_means:
      "A hipercoagulabilidade é raramente diagnosticada por INR baixo. Ingestão recente de vitamina K. Geralmente não é uma descoberta sinalizada.",
  },
  aptt: {
    description:
      "Via de coagulação intrínseca + comum. Utilizado para monitorizar heparina não fracionada e rastreio de deficiências de fator VIII/IX/XI/XII e anticoagulante lúpico.",
    high_means:
      "Terapêutica com heparina (alvo ~1,5–2,5× controlo), hemofilia A/B, DVW, DIC, doença hepática grave, anticoagulante lúpico (paradoxal – pró-trombótico), anticoagulantes orais diretos em doses elevadas.",
    low_means:
      "Elevações do fator VIII na fase aguda podem encurtar o aPTT – inespecífico.",
    unit_notes:
      "Referência 25–35 segundos (dependendo do laboratório). HNF terapêutica 1,5–2,5× limite superior.",
  },
  fibrinogenio: {
    high_means:
      "Inflamação, infeção, malignidade, gravidez, contracetivos orais, tabagismo. Muito elevado (>700) associado a risco trombótico.",
    description:
      "Fator de coagulação I – precursor da fibrina. Reagente de fase aguda. Valores baixos indicam consumo (DIC, doença hepática grave) ou deficiência genética; valores elevados são marcadores de inflamação inespecíficos.",
    low_means:
      "DIC (consumida), doença hepática grave (subproduzida), afibrinogenemia/disfibrinogenemia congénita, terapêutica trombolítica. <100 mg/dL = risco de hemorragia.",
  },
  ferro_serico: {
    description:
      "Ferro sérico – ferro circulante ligado à transferrina. Altamente variável (diurno, pós-refeição, inflamação), pelo que sempre interpretado com transferrina/TSAT/ferritina.",
    high_means:
      "Sobrecarga de ferro (hemocromatose transfusional), suplementação de ferro, hemólise, doença hepática, intoxicação por chumbo. Um único valor elevado raramente é acionável.",
    low_means:
      "Deficiência de ferro, doença crónica (bloqueio funcional do ferro), hemorragia recente. Combinar com transferrina (alta) e ferritina (baixa) para distinguir da anemia de inflamação crónica.",
  },
  transferrina: {
    description:
      "Proteína transportadora de ferro sintetizada pelo fígado. Regulado inversamente pelas reservas de ferro – aumento da deficiência de ferro, queda da inflamação/insuficiência hepática/malnutrição.",
    high_means:
      "Deficiência de ferro (mais comum), gravidez, contracetivos orais.",
    low_means:
      "Anemia de doença crónica (a inflamação suprime a síntese), doença hepática grave, malnutrição, síndrome nefrótico (perda urinária).",
  },
  saturacao_transferrina: {
    description:
      "Ferro / TIBC × 100. O melhor rastreio para o estado de ferro juntamente com a ferritina: baixa deficiência de ferro (<20%, frequentemente <10%), elevada sobrecarga de ferro (>45% sugestiva, >55% nas mulheres / >60% nos homens = rastreio de hemocromatose positivo).",
    high_means:
      "Hemocromatose hereditária (>55% f / >60% m repetida), sobrecarga transfusional, suplementação, hemólise, doença hepática.",
    low_means:
      "Deficiência de ferro. Juntamente com baixa ferritina, praticamente diagnóstico. A anemia da doença crónica apresenta geralmente um TSAT baixo, mas ferritina normal/alta.",
  },
  tibc: {
    description:
      "Capacidade total de ligação ao ferro — proxy para a transferrina (≈ transferrina × 1,4). Elevado em deficiência de ferro, baixo em doenças crónicas/insuficiência hepática/malnutrição.",
    high_means: "Deficiência de ferro, gravidez tardia, contracetivos orais.",
    low_means:
      "Anemia de doença crónica, malnutrição, doença hepática/renal grave, estados inflamatórios.",
  },
  bilirrubina_direta: {
    description:
      "Bilirrubina conjugada – solúvel em água, processada pelo fígado e excretada pela bílis. A elevação indica disfunção hepatocelular ou obstrução biliar.",
    high_means:
      "Colestase, obstrução biliar, hepatite, lesão hepática induzida por medicamentos, síndrome de Dubin-Johnson/Rotor (raro).",
    low_means: "Não clinicamente significativo quando baixo.",
    unit_notes:
      "Referência <0,3 mg/dL. Fração direta >50% da bilirrubina total = padrão pós-hepático/colestático.",
  },
  bilirrubina_indireta: {
    description:
      "Bilirrubina não conjugada – produto de degradação do heme antes do processamento hepático. Calculado como Total - Directo.",
    high_means:
      "Hemólise (mais comum), síndrome de Gilbert (muito comum, benigna), Crigler-Najjar (raro), reabsorção de hematoma, eritopoiese ineficaz.",
    low_means: "Não clinicamente significativo.",
  },
  amilase: {
    description:
      "Enzima pancreática + salivar que hidrolisa os hidratos de carbono. Marcador clássico de pancreatite – aumenta em 6–12h, atinge o pico em 24–48h, desce em 3–5 dias. Menos específico que a lipase.",
    high_means:
      "Pancreatite aguda (frequentemente >3× limite superior), cancro do pâncreas, doença salivar (papeira, parotidite), obstrução/perfuração intestinal, gravidez ectópica, insuficiência renal (depuração reduzida), macroamilasemia (benigna).",
    low_means:
      "Pancreatite crónica com destruição pancreática significativa, doença hepática grave, fibrose quística.",
  },
  lipase: {
    high_means:
      "Pancreatite aguda/crónica (diagnóstico >3× LSN), cancro do pâncreas, obstrução/perfuração intestinal, insuficiência renal, uso de opióides.",
    description:
      "Enzima hidrolisante lipídica específica do pâncreas. Mais sensível E mais específico que a amilase para a pancreatite. Aumenta 4–8h, atinge o pico 24h, persiste 8–14 dias.",
    low_means: "Pancreatite crónica com danos extensos, fibrose quística.",
  },
  haptoglobina: {
    description:
      "Proteína de fase aguda que se liga à hemoglobina livre. O teste mais útil para a hemólise intravascular: consumido quando a Hb livre é libertada (baixa ou indetetável).",
    low_means:
      "Hemólise (intravascular > extravascular), doença hepática grave, ahaptoglobinemia congénita (~4% da população).",
    high_means:
      "Inflamação, infecção, malignidade, terapêutica com glicocorticóides. Aumento da fase aguda.",
    unit_notes:
      "Referência 30–200 mg/dL. Parear com LDH (elevado), bilirrubina indireta (elevado), reticulócitos (elevado) e esfregaço periférico em caso de suspeita de hemólise.",
  },
  nt_probnp: {
    description:
      "Fragmento clivado de peptídeo natriurético tipo pró-B, libertado dos miócitos cardíacos em resposta ao estiramento da parede ventricular. Teste padrão de exclusão para insuficiência cardíaca na avaliação da dispneia.",
    low_means:
      "Insuficiência cardíaca essencialmente excluída em doentes sintomáticos de ambulatório (<125 pg/mL) ou em quadro agudo (<300 pg/mL).",
    high_means:
      "Insuficiência cardíaca aguda descompensada (> 900 em <50 anos, > 1.800 em 50-75 anos, > 1.800 em > 75 anos normalmente), IC crónica, IM, fibrilhação auricular, insuficiência renal, idade avançada. Maior com FE reduzida mais que preservada.",
    unit_notes:
      "Pontos de corte estratificados por idade: <50 anos >450, 50-75 anos >900, >75 anos >1800 pg/mL. A obesidade baixa valores; a doença renal aumenta-os.",
  },
  ck_mb: {
    high_means:
      "Enfarte do miocárdio (aumento às 4–6h, pico às 12–24h), miocardite, cirurgia cardíaca, lesão muscular grave (CK-MB esquelética ~1–3% da CK total).",
    low_means: "Não clinicamente significativo.",
    description:
      "Isoenzima de creatina quinase enriquecida para o coração. Atualmente amplamente substituída pela troponina para diagnóstico de enfarte do miocárdio, mas ainda utilizada para detetar o reinfarto (resolve em 36-48 horas) e quantificar a lesão cardíaca perioperatória.",
  },
  mioglobina: {
    high_means:
      "IM (precoce), rabdomiólise (muito elevado — risco de nefropatia pigmentar), traumatismo, distrofia muscular, exercício intenso, injeção intramuscular, abuso de álcool.",
    low_means: "Não clinicamente significativo.",
    description:
      "Proteína heme no músculo esquelético e cardíaco. Marcador cardíaco mais precoce (aumenta 1–3 horas após a lesão), mas muito pouco específico – qualquer lesão muscular aumenta. Em grande parte substituído por hs-troponina.",
  },
  cistatina_c: {
    description:
      "Proteína de baixo peso molecular filtrada livremente no glomérulo e reabsorvida pelas células tubulares. Independentemente da massa muscular — melhor estimador da TFG do que a creatinina em doentes idosos, frágeis ou musculados.",
    low_means: "Geralmente não sinalizado.",
    high_means:
      "Filtração glomerular reduzida. Menos afetado pelos músculos, dieta e raça do que a creatinina. Utilizado em cistatina CKD-EPI e equações combinadas de creatinina-cistatina para TFG refinada.",
    unit_notes:
      "Referência 0,6–1,0 mg/L (dependente do ensaio). Aumenta com lesão renal; também ligeiros aumentos com esteróides, hipertiroidismo.",
  },
  microalbuminuria: {
    high_means:
      "Nefropatia diabética, doença renal hipertensiva, glomerulonefrite, pré-eclâmpsia. 30–300 mg/24h = moderadamente aumentado (anteriormente microalbuminúria); >300 = gravemente aumentado (proteinúria evidente).",
    description:
      "Excreção urinária de albumina – o primeiro sinal detetável de doença renal diabética/hipertensiva. A relação spot albumina/creatinina (ACR) é a preferida (não é necessária colheita de 24 horas).",
    low_means: "Normal – barreira renal intacta.",
  },
  rac: {
    description:
      "Proporção spot de albumina/creatinina na urina — preferível à colheita de 24 horas para rastreio de albuminúria e estadiamento da DRC (limiares KDIGO A1/A2/A3: <30 / 30–300 / >300 mg/g).",
    low_means:
      "A1 (<30): albuminúria normal – não foi detetado qualquer dano renal.",
    high_means:
      "A2 (30–300): nefropatia diabética/hipertensiva precoce. A3 (>300): nefropatia manifesta, doença glomerular.",
  },
  t3_total: {
    description:
      "Triiodotironina total (ligada + livre) – a hormona tiroideia ativa. Menos comummente ordenado que o FT3 porque as medições totais são fortemente influenciadas pelas alterações nas proteínas de ligação.",
    high_means:
      "Hipertiroidismo (graves, nódulo tóxico), toxicose por T3, terapêutica exógena com T3, gravidez/estrogénios (TBG up), algumas doenças malignas.",
    low_means:
      "Hipotiroidismo (evidente), doença não tiroideia grave (síndrome de T3 baixo – mais comum), fome, medicamentos (amiodarona, glicocorticóides).",
  },
  ft3: {
    high_means:
      "Hipertiroidismo evidente (Graves), Graves com predominância de T3, nódulo tóxico, tirotoxicose factícia (ingestão de T3).",
    low_means:
      "Hipotiroidismo (evidente), síndrome de T3 baixo (eutiroideu doente), hospitalização/doença crítica.",
    description:
      "T3 livre e biologicamente ativo. Utilizado para confirmar o hipertiroidismo (especialmente a toxicose por T3, em que o T4L pode ser normal) e monitorizar a terapêutica de substituição de T3.",
  },
  t4_total: {
    low_means: "Hipotiroidismo, doença grave, síndrome nefrótico (TBG baixa).",
    high_means:
      "Hipertiroidismo, gravidez/estrogénios (TBG up), hipertiroxinemia disalbuminémica familiar, internamento psiquiátrico agudo (transitório).",
    description:
      "Tiroxina total (ligada + livre). Em grande parte substituído pelo FT4 – os valores totais são confundidos por alterações na TBG (estrogénios, gravidez, doença hepática).",
  },
  anti_tpo: {
    high_means:
      "Tiroidite de Hashimoto (hipotiroidismo autoimune), doença de Graves, tiroidite pós-parto, diabetes tipo 1 (associada). Título elevado + hipotiroidismo subclínico prediz fortemente a progressão para doença evidente.",
    low_means:
      "Negativo – doença autoimune da tiroide improvável (mas não descarta).",
    description:
      "Anticorpos contra a peroxidase tiroideia – presentes em mais de 90% da tiroidite de Hashimoto e em aproximadamente 75% da tiroidite de Graves. Preditor mais forte de progressão de hipotiroidismo subclínico para hipotiroidismo evidente.",
    unit_notes:
      "Referência <35 UI/mL (dependente do ensaio – também reportado como <60 em alguns métodos). O título correlaciona-se com a atividade da doença na doença de Hashimoto.",
  },
  anti_tg: {
    high_means:
      "Hashimoto, Graves, doença autoimune da tiroide. No cancro da tiroide tratado: a presença interfere na dosagem da tiroglobulina (deve ser referida ao lado).",
    description:
      "Anticorpos contra a tiroglobulina – comarcador de doenças autoimunes da tiroide. Mais útil na monitorização do cancro diferenciado da tiroide pós-tiroidectomia: a presença interfere na medição de TG.",
    low_means:
      "Negativo – não interfere ativamente no rastreio dos marcadores tumorais da tiroglobulina.",
  },
  tireoglobulina: {
    description:
      "Glicoproteína produzida exclusivamente pelas células foliculares da tiroide. Marcador tumoral primário para seguimento de cancro diferenciado da tiroide pós-tiroidectomia + ablação com radioiodo.",
    high_means:
      "Pós-tiroidectomia: cancro diferenciado da tiroide residual/recorrente/metastático. Pré-tratamento: qualquer patologia da tiroide (inespecífica) — não utilizada como diagnóstico de cancro.",
    unit_notes:
      "Solicite sempre com anticorpo anti-TG (interfere no ensaio se positivo).",
    low_means:
      "Após tiroidectomia total + ablação: indetetável significa ausência de tecido tiroideu funcional (idealmente <0,2 ng/mL). Indica remissão.",
  },
  pth: {
    description:
      "Hormona paratiroide — regulador primário de cálcio/fosfato/vitamina D. Sempre interpretado com cálcio e vitamina D simultâneos.",
    unit_notes:
      "Referência 15–65 pg/mL (ensaio de PTH intacto). Variação diurna - idealmente, colete AM.",
    low_means:
      "Hipoparatiroidismo (Ca baixo + PTH baixo = falha na resposta apropriada), hipercalcemia maligna (Ca alto, PTH suprimido), hipomagnesemia.",
    high_means:
      "Hiperparatiroidismo primário (Ca elevado, PTH elevado, P baixo), hiperparatiroidismo secundário (DRC, deficiência de vit D – Ca baixo/normal, PTH elevado), terciário (autónomo após secundário de longa duração).",
  },
  insulina: {
    description:
      "Hormona das células β pancreáticas. Utilizado para avaliar hipoglicemia (insulinoma), resistência à insulina (HOMA-IR) e tumores raros de células β. Apenas interpretação do estado de jejum.",
    high_means:
      "Resistência à insulina, diabetes tipo 2 (precoce - insulina elevada + glicose elevada), insulinoma (insulina elevada + glicose baixa), insulina exógena (distingue a pró-insulina elevada), doença de Cushing, acromegalia, obesidade.",
    low_means:
      "Diabetes tipo 1 (destruição das células β), tipo 2 tardio (exaustão das células β), pancreatectomia, pancreatite grave.",
  },
  homa_ir: {
    low_means:
      "Altamente sensível à insulina (atlético) ou insuficiência das células β (tipo 1, tipo 2 tardio com insulina baixa).",
    high_means:
      "Resistência à insulina: <1,0 normal, 1,0–2,5 limítrofe, >2,5 resistente, >5 grave. SOP, síndrome metabólica, diabetes tipo 2, DHGNA.",
    description:
      "Avaliação do modelo homeostático de resistência à insulina: (insulina em jejum × glicemia em jejum) / 405. Proxy rápido e validado para a sensibilidade à insulina em avaliação metabólica / SOP / NAFLD.",
  },
  c_peptideo: {
    low_means:
      "Diabetes tipo 1 (destruição das células β), tipo 2 tardio, pancreatectomia. Hipoglicemia factícia por insulina exógena: peptídeo C baixo + insulina alta.",
    description:
      "Dividido da pró-insulina na proporção de 1:1 com insulina. Reflete a produção endógena de insulina (ao contrário da insulina medida, que pode incluir exógena). Distingue a diabetes tipo 1 (baixa) do tipo 2 (normal/alta) e deteta hipoglicemia factícia.",
    high_means:
      "Resistência à insulina/diabetes tipo 2, insulinoma, uso de sulfonilureia (aumenta a insulina e o péptido C), insuficiência renal (depuração reduzida).",
  },
  progesterona: {
    low_means:
      "Anovulação (sem aumento lúteo), defeito da fase lútea, menopausa, agonistas estrogénios/GnRH exógenos.",
    high_means:
      "Ovulação (pico lúteo médio), gravidez (aumenta com a idade gestacional), hiperplasia congénita da supra-renal (deficiência de 21-hidroxilase).",
    description:
      "Hormona progestacional. Confirma a ovulação (lúteo médio >3 ng/mL), monitoriza o tratamento de fertilidade, apoia o diagnóstico precoce da gravidez (>20 = viável).",
    unit_notes:
      "Dependente da fase do ciclo. Pico lúteo médio (~ dia 21 do ciclo de 28 dias) >3 ng/mL confirma a ovulação; >10 = função lútea robusta.",
  },
  amh: {
    high_means:
      "SOP (frequentemente >5–8), tumor das células da granulosa (raro). Maior AMH = mais folículos antrais disponíveis.",
    description:
      "Hormona das células da granulosa – melhor marcador único da reserva ovárica feminina. Estável durante todo o ciclo menstrual (ao contrário da FSH). Utilizado no planeamento de fertilização in vitro e na investigação de SOP.",
    low_means:
      "Reserva ovárica diminuída, abordagem da menopausa, pós-quimioterapia/radioterapia, cirurgia ovárica. <0,5 = reserva muito baixa, resposta de fertilização in vitro provavelmente fraca.",
    unit_notes:
      "Mulheres em idade fértil 1,0–9,0 ng/mL; declínio dependente da idade (desce cerca de 50% por década após os 30). Pós-menopausa: indetetável.",
  },
  beta_hcg: {
    description:
      "Hormona da gravidez produzida pelo trofoblasto – também um marcador tumoral para as células germinativas e doenças trofoblásticas. Duplica a cada aproximadamente 48 horas no início da gravidez viável.",
    low_means:
      "Não grávida ou pós-gravidez. <5 mUI/mL exclui a gravidez na maioria dos contextos.",
    high_means:
      "Gravidez (crescente), gravidez molar/coriocarcinoma (muito elevado), tumores de células germinativas (não-seminoma testicular), algumas doenças malignas não trofoblásticas. Valores em queda no início da gravidez: aborto espontâneo/ectópico.",
    unit_notes:
      "Não grávida <5. Soro quantitativo mais sensível que a urina. Tempo de duplicação 48–72h em gravidez viável <8 semanas.",
  },
  dhea_s: {
    description:
      "Forma sulfatada de DHEA – androgénio supra-renal produzido quase exclusivamente pelo córtex supra-renal (zona reticular). Estável (t½ longo), pelo que a medição única reflete a produção de androgénios adrenais.",
    high_means:
      "Hiperplasia adrenal congénita (21-OH ou 11-OH), tumor adrenal (muito elevado – virilização), Cushing, SOP (leve), suplementação exógena de DHEA.",
    low_means:
      "Insuficiência adrenal, hipopituitarismo, terapêutica com glicocorticóides (supressão), declínio relacionado com a idade (“adrenopausa” – queda de aproximadamente 10%/década após os 30 anos).",
    unit_notes:
      "Fortemente dependente da idade e do sexo. Pico feminino 25 anos ~430, pós-menopausa ~50. Pico masculino ~30y ~560, declínio depois disso.",
  },
  _17_oh_progesterona: {
    high_means:
      "HAC clássica (muito elevada, frequentemente >100 em recém-nascidos), HAC de início tardio (ligeiramente elevada, confirma o teste de estimulação com ACTH), tumor da supra-renal.",
    description:
      "Esteróide intermédio na síntese de cortisol. Teste de rastreio para hiperplasia adrenal congénita por deficiência de 21-hidroxilase (forma mais comum).",
    low_means: "Geralmente não significativo.",
    unit_notes:
      "Referência <2,0 ng/mL (200 ng/dL). >10 ng/mL sugere HAC; Segue-se o teste de diagnóstico com ACTH-stim.",
  },
  iga: {
    description:
      "Imunoglobulina predominante na superfície da mucosa (intestino, respiratória, GU) e secreções. A deficiência seletiva de IgA é a imunodeficiência primária mais comum (1:300 – frequentemente assintomática; pode complicar as transfusões e os testes serológicos de celíacos com IgA anti-tTG).",
    high_means:
      "Mieloma múltiplo (tipo IgA), doença hepática, infeção crónica, doença autoimune, nefropatia por IgA.",
    low_means:
      "Deficiência seletiva de IgA (mais comum – frequentemente incidental), ICV, imunossupressão. Se estiver a verificar IgA anti-tTG celíaca, deve-se primeiro excluir a deficiência de IgA.",
  },
  igg: {
    description:
      "Imunoglobulina sérica mais abundante – medeia a resposta imunitária secundária e atravessa a placenta. IgG quantitativa utilizada na investigação de imunodeficiência, deteção de gamopatia monoclonal e monitorização de terapêutica de substituição.",
    high_means:
      "Policlonal: infeção crónica, doença autoimune, doença hepática crónica. Monoclonal: mieloma múltiplo, MGUS, Waldenström. Doença relacionada com IgG4 (subclasse IgG4).",
    low_means:
      "CVID, agamaglobulinemia de Bruton, terapêutica imunossupressora, síndrome nefrótico, enteropatia grave com perda de proteínas. Infeções recorrentes por bactérias encapsuladas.",
  },
  igm: {
    description:
      "Primeiro anticorpo produzido na resposta imunitária primária. Estrutura pentamérica — aglutina eficazmente. A serologia para infecção aguda detecta IgM como evidência de exposição recente.",
    high_means:
      "Infeção aguda/recente, macroglobulinemia de Waldenström (muito elevada), colangite biliar primária (tipicamente elevada), algumas infeções crónicas.",
    low_means:
      "Síndrome hiper-IgM (falha na mudança de classe), ICV, amiloidose de cadeia leve.",
  },
  c3: {
    low_means:
      "Flare ativo do LES, GN pós-estreptocócica, GNMP, SHU atípica, crioglobulinemia, deficiência congénita. C3 baixo + C4 normal: ativação de via alternativa.",
    description:
      "Componente central do complemento, consumido pela ativação das vias clássica e alternativa. C3 baixo + C4 baixo = LES ativo; C3 isolado baixo sugere GN pós-infecciosa, SHU atípica, GNMP.",
    high_means:
      "Reagente de fase aguda: inflamação, infeção, malignidade. Raramente a pista principal.",
  },
  c4: {
    high_means: "Reagente de fase aguda.",
    description:
      "Componente do complemento da via clássica. C4 baixo + C3 baixo = exacerbação do LES ou angioedema hereditário; C4 isolado baixo sugere crioglobulinemia ou AEH.",
    low_means:
      "LES (ativo), angioedema hereditário (muito baixo — diagnóstico com baixa função de C1-INH), deficiência adquirida de C1-INH, crioglobulinemia, deficiência congénita.",
  },
  ana: {
    description:
      "Imunofluorescência indireta em células Hep-2 — teste de rastreio para doenças autoimunes sistémicas (LES, doença de Sjögren, esclerodermia, DMTC, polimiosite). Elevada sensibilidade, baixa especificidade (5–15% da população saudável com título baixo positivo).",
    high_means:
      "Título + guia de padrão para investigação adicional: ≥1:80 com sintomas relevantes garante dsDNA, Sm, RNP, SSA/B, Scl-70, etc. Título mais elevado = mais clinicamente significativo.",
    low_means:
      "Negativo – doença autoimune não excluída clinicamente, mas etiologia imunomediada menos provável.",
    unit_notes:
      "Reportado como título + padrão (homogéneo, salpicado, nucleolar, centrómero, etc.). 1:40 fracamente positivo, 1:80–160 limítrofe, ≥1:320 título elevado.",
  },
  factor_reumatoide: {
    description:
      "Autoanticorpo (normalmente IgM) contra a porção Fc da IgG. Sensível, mas não específico para a artrite reumatóide (~70-80% sens, 80% spec). Aumenta também na doença de Sjögren, hepatite C, infeções crónicas e idosos saudáveis.",
    high_means:
      "Artrite reumatóide (título elevado = doença mais agressiva), doença de Sjögren, doença mista do tecido conjuntivo, crioglobulinemia, hepatite C crónica, endocardite, idosos saudáveis ​​(~5-10%).",
    low_means:
      "AR não excluída – considerar anti-CCP (mais específico). Existe AR seronegativa.",
  },
  anti_ccp: {
    description:
      "Anticorpos contra péptidos citrulinados cíclicos — o marcador serológico mais específico (~95%) e sensível (~70%) para a artrite reumatóide. Prevê doença erosiva/agressiva.",
    high_means:
      "Artrite reumatóide (título alto = agressivo, erosivo). Pode anteceder a doença clínica em anos.",
    low_means:
      "A RA é muito menos provável. A AR negativa para anti-CCP existe, mas é mais ligeira.",
  },
  psa_total: {
    description:
      "Glicoproteína específica da próstata. Marcador de rastreio/monitorização do cancro da próstata; aumenta também na HBP, prostatite, ejaculação recente, instrumentação.",
    unit_notes:
      "Ajustado por idade: <2,5 (40-49), <3,5 (50-59), <4,5 (60-69), <6,5 (70-79). A proporção de PSA livre/total refina HBP versus cancro (proporção baixa = preocupação com o cancro).",
    high_means:
      "Cancro da próstata, HBP, prostatite, biópsia/cateterismo/ejaculação recente, idade avançada. A velocidade (>0,75 ng/mL/ano) e a densidade (PSA/volume da próstata) refinam a suspeita de cancro.",
    low_means:
      "Menor probabilidade de cancro. Suprimido pelos inibidores da 5α-redutase (finasterida/dutasterida — valores reais × 2 para comparar).",
  },
  psa_livre: {
    high_means: "Maior fracção livre → mais provável HBP.",
    description:
      "Fração não consolidada (livre) de PSA. Utilizado como rácio (% livre/total) quando o PSA total é de 4–10 ng/mL para discriminar a HBP (% livre alto) do cancro (% livre baixo).",
    low_means:
      "Proporção livre/total <10% sugere fortemente cancro; 10–25% intermédio; >25% favorece a HBP.",
  },
  afp: {
    description:
      "Proteína hepática fetal – marcador tumoral primário para carcinoma hepatocelular (CHC) e tumores do saco vitelino/misto de células germinativas. Utilizado na vigilância do CHC na cirrose, monitorizando o cancro testicular.",
    low_means:
      "Normal em adultas não grávidas. <10 ng/mL exclui a maioria dos tumores de células germinativas, mas não apenas o CHC.",
    high_means:
      "CHC (frequentemente >400 em fase avançada; pode ser normal em fase inicial), tumores de células germinativas (não seminoma), hepatite/cirrose (modesta), gravidez (muito elevada — fisiológica). Tendência crescente > o valor absoluto é importante na vigilância.",
  },
  cea: {
    description:
      "Glicoproteína expressa no intestino fetal e em muitos adenocarcinomas. Mais útil para monitorizar (não rastrear) o cancro colorretal pós-ressecção – valores crescentes sugerem recorrência.",
    high_means:
      "Carcinoma colorretal, gástrico, pancreático, pulmonar, mamário e medular da tiroide. Fumador (~5–10), doença inflamatória intestinal, pancreatite, DPOC, hipotiroidismo, doença hepática.",
    low_means: "Normal – não exclui a malignidade.",
    unit_notes:
      "Referência <5 ng/mL não fumador, <10 fumador. A tendência ao longo do tempo é mais útil do que absoluta.",
  },
  ca_19_9: {
    description:
      "Antigénio Lewis-a sialilado. Marcador tumoral primário para o cancro do pâncreas; aumenta também no colangiocarcinoma, obstrução biliar (qualquer causa). 5–10% da população é Lewis-negativa – não atingirá CA 19-9 mesmo com cancro.",
    high_means:
      "Adenocarcinoma pancreático (frequentemente muito elevado — >1000 avançado), colangiocarcinoma, cancro gástrico/colorretal, obstrução biliar (qualquer), pancreatite, cirrose.",
    low_means:
      "Menor probabilidade de cancro. Indivíduos Lewis-negativos falso-baixo.",
  },
  ca_125: {
    high_means:
      "Cancro epitelial do ovário, cancro do endométrio, inflamação peritoneal/pleural/pericárdica (derrame de qualquer causa), endometriose, miomas, gravidez no primeiro trimestre, menstruação, pancreatite.",
    low_means:
      "Normal – não exclui o cancro do ovário (especialmente em fase inicial).",
    description:
      "Glicoproteína mucinosa. Marcador tumoral primário para vigilância do cancro epitelial do ovário e resposta ao tratamento. Muitos falsos positivos em condições ginecológicas benignas.",
    unit_notes:
      "Referência <35 U/mL. Utilize a tendência ao longo do tempo para vigilância. As pontuações ROMA/RMI combinam-se com HE4/ecografia para o risco de massa anexial.",
  },
  ca_15_3: {
    high_means:
      "Cancro da mama metastático, outros adenocarcinomas, doenças benignas da mama/fígado/pulmão.",
    low_means: "Normal – não exclui o cancro da mama.",
    description:
      "Marcador de vigilância do cancro da mama (não rastreio). Aumenta em aproximadamente 75% dos casos de cancro da mama metastático; menos fiável no início da doença.",
  },
  ctx: {
    description:
      "Marcador de reabsorção óssea — produto de degradação do colagénio tipo I libertado pela atividade dos osteoclastos. Utilizado para monitorizar a terapêutica antirreabsortiva (bifosfonatos, denosumab) — cai 30–70% em 3 meses com terapêutica eficaz.",
    high_means:
      "Elevada renovação óssea: osteoporose não tratada, hipertiroidismo, hiperparatiroidismo, doença de Paget, malignidade (metástases ósseas), fratura recente.",
    unit_notes:
      "Variação diurna (pico matinal), preferencialmente colheita em jejum. A comparação pré/pós-terapia é mais útil.",
    low_means:
      "Terapêutica anti-reabsortiva eficaz, hipoparatiroidismo. A CTX suprimida (<150 pg/mL) com bifosfonatos correlaciona-se com um risco elevado de fratura atípica durante o uso prolongado.",
  },
  p1np: {
    description:
      "Marcador de formação óssea – libertado durante a síntese de colagénio tipo I pelos osteoblastos. Utilizado para monitorizar a terapêutica anabólica (teriparatida, abaloparatida, romosozumab), onde deve aumentar dentro de 1–3 meses.",
    high_means:
      "Formação óssea: agentes anabolizantes, consolidação de fraturas recentes, doença de Paget, hiperparatiroidismo, crescimento (crianças).",
    low_means:
      "Utilização de glicocorticóides, terapêutica pós-antirreabsortiva (alguns), hipoparatiroidismo.",
  },
  osteocalcina: {
    description:
      "Marcador de formação óssea — proteína não colagenosa sintetizada pelos osteoblastos. Dependente de vitamina K. Utilizado como marcador de pesquisa/segunda linha para avaliação da remodelação óssea.",
    low_means:
      "Terapêutica com glicocorticóides, hipotiroidismo, hipoparatiroidismo.",
    high_means:
      "Elevada renovação óssea: hiperparatiroidismo, hipertiroidismo, doença de Paget, osteomalácia, insuficiência renal (depuração prejudicada).",
  },
  vitamina_a: {
    description:
      "Vitamina lipossolúvel essencial para a visão (rodopsina), função imunitária, integridade epitelial e desenvolvimento embrionário.",
    high_means:
      "Hipervitaminose A: consumo excessivo crónico (suplementos/fígado), hepatotoxicidade. Toxicidade aguda (>200) — pseudotumor cerebral, descamação da pele, teratogénico.",
    low_means:
      "Má absorção (fibrose quística, DII, pós-bariátrica), desnutrição proteica grave, alcoolismo. Provoca cegueira noturna, xeroftalmia e aumento do risco de infeção.",
  },
  vitamina_e: {
    low_means:
      "Má absorção de gordura (FC, abetalipoproteinemia, colestase), desnutrição grave. Provoca neuropatia (ataxia espinho-cerebelosa, arreflexia), anemia hemolítica.",
    high_means:
      "Suplementação excessiva. Pode aumentar o risco de hemorragia com varfarina.",
    description:
      "Antioxidante solúvel em gordura – protege as membranas celulares da peroxidação lipídica. Deficiência rara em adultos bem nutridos; clinicamente relevante em distúrbios de má absorção de gordura.",
  },
  vitamina_b1: {
    description:
      "Co-factor para o metabolismo dos hidratos de carbono (piruvato desidrogenase, transcetolase). Deficiência: beribéri (húmido – insuficiência cardíaca; seco – neuropatia), Wernicke-Korsakoff em alcoólicos.",
    high_means: "Suplementação excessiva. Raramente tóxico.",
    low_means:
      "Alcoolismo (mais comum), desnutrição, hiperemese gravídica, pós-bariátrica, síndrome de realimentação (consumida). Abasteça-se sempre antes de glicose em caso de suspeita de deficiência.",
  },
  vitamina_b6: {
    description:
      "Co-factor para o metabolismo dos aminoácidos, síntese de neurotransmissores, síntese de hemoglobina. A PLP é a forma ativa da coenzima.",
    high_means:
      "Toxicidade da suplementação (>1000 nmol/L) — neuropatia sensorial paradoxal.",
    low_means:
      "Alcoolismo, terapêutica com isoniazida/hidralazina/penicilamina, diálise renal, má absorção. Provoca anemia sideroblástica, dermatite, glossite, neuropatia periférica.",
  },
  zinco: {
    description:
      "Oligoelemento essencial — co-factor para >300 enzimas. Necessário para a função imunitária, cicatrização de feridas, crescimento, paladar/olfato. Níveis séricos variáveis ​​​​(fase aguda, jejum, hemólise).",
    high_means:
      "Fase aguda de inflamação (rara), suplementação, hemólise (artefactual – glóbulos vermelhos ricos em Zn).",
    low_means:
      "Acrodermatite enteropática (genética), má absorção, alcoolismo, gravidez, anemia falciforme, doença crónica, nutrição parentérica sem suplementação. Causa queda de cabelo, erupção cutânea, alteração do paladar e disfunção imunitária.",
  },
  cobre: {
    description:
      "Oligoelemento essencial - co-factor para a ceruloplasmina, citocromo c oxidase, lisil oxidase, dopamina β-hidroxilase. Indicação primária de investigação da doença de Wilson.",
    high_means:
      "Gravidez/contracetivos orais (o estrogénio aumenta a ceruloplasmina), inflamação, hipertiroidismo. A doença de Wilson tem cobre sérico variável (habitualmente normal – necessita de ceruloplasmina + 24h de cobre urinário).",
    low_means:
      "Doença de Wilson (normalmente), doença de Menkes (genética), desnutrição, má absorção, suplementação excessiva de zinco, nutrição parentérica sem cobre. Provoca anemia, neutropenia, neurodegeneração.",
  },
  ceruloplasmina: {
    description:
      "Reagente de fase aguda portador de cobre. Ceruloplasmina baixa + cobre urinário alto + cobre sérico baixo = doença de Wilson.",
    low_means:
      "Doença de Wilson (frequentemente <10), Menkes, desnutrição proteica grave, síndrome nefrótico.",
    high_means: "Inflamação, infeção, gravidez/estrogénios, malignidade.",
  },
  apo_b: {
    low_means:
      "Terapêutica com estatinas/PCSK9/ezetimiba (alvo), hipertiroidismo, doença hepática grave, abetalipoproteinemia (genética).",
    description:
      "Proteína estrutural em cada partícula de lipoproteína aterogénica (LDL, VLDL, IDL, Lp(a)). Conta todas as partículas potencialmente aterogénicas 1:1, independentemente do teor de colesterol. Cada vez mais preferido ao LDL-C para a avaliação do risco CV.",
    high_means:
      "Aumento da contagem de partículas aterogénicas: hipercolesterolemia familiar, diabetes tipo 2/síndrome metabólica (LDL pequeno e denso), hipotiroidismo, síndrome nefrótica.",
    unit_notes:
      "Objectivos: <100 prevenção primária, <80 prevenção de alto risco, <65 prevenção secundária. Discordância ApoB/LDL-C sugere padrão sdLDL.",
  },
  lp_a: {
    description:
      "Partícula semelhante a LDL geneticamente determinada ligada covalentemente à apolipoproteína(a). Fator de risco CV independente, especialmente estenose aórtica. Medição única recomendada.",
    low_means: "Menor risco CV desta via.",
    high_means:
      "Genético – risco independente de CV/estenose aórtica. >50 mg/dL (>125 nmol/L) elevado; >90 mg/dL elevado. As estatinas NÃO diminuem; aférese de lipoproteínas ou terapias específicas (em desenvolvimento) necessárias.",
  },
  frutosamina: {
    description:
      "Proteínas séricas glicadas (principalmente albumina) – refletem a glicose média nas 2–3 semanas anteriores. Útil quando a HbA1c não é fiável: hemoglobinopatias, transfusão recente, anemia hemolítica, doença renal terminal, gravidez.",
    high_means:
      "Hiperglicemia nas últimas 2–3 semanas. Níveis: 205–285 normais, 286–320 diabetes bem controlada, 321–370 limítrofes, >370 controlo deficiente.",
    low_means:
      "Hipoalbuminemia (síndrome nefrótica, doença hepática grave, desnutrição – falsamente baixa), hipertiroidismo.",
  },
  procalcitonina: {
    description:
      "Precursor da calcitonina libertado especificamente por infecção bacteriana (induzida pela endotoxina/IL-6/TNF-α). Cresce mais rapidamente e é mais específico para as bactérias do que a PCR. Utilizado para discriminar a etiologia bacteriana versus viral/inflamatória e para orientar a administração de antibióticos.",
    unit_notes:
      "<0,25 infeção bacteriana improvável · 0,25–0,5 possível · 0,5–2 provável · >2 elevado risco de sépsis.",
    low_means:
      "Provável infeção viral ou inflamação não infeciosa. Valor preditivo negativo útil para a sépsis bacteriana em <0,25 ng/mL.",
    high_means:
      "Sepse bacteriana (>2 = alto risco, >10 = sepse grave/choque séptico), pneumonia bacteriana. Além disso: carcinoma medular da tiroide, paraneoplásico, cirurgia/traumatismo/queimaduras recentes, insuficiência renal (leve).",
  },
  calprotectina_fecal: {
    low_means:
      "<50 µg/g exclui praticamente a atividade da DII – a SII é a explicação mais provável para os sintomas gastrointestinais crónicos.",
    description:
      "Proteína derivada dos neutrófilos libertada no lúmen intestinal durante a inflamação intestinal. Discrimina a doença inflamatória intestinal da síndrome do intestino irritável e rastreia a atividade da DII/resposta ao tratamento.",
    high_means:
      "A DII ativa (>250 sugere fortemente doença ativa), gastroenterite bacteriana, enteropatia por AINE, cancro colorretal (modesto), pólipos. Tendência mais útil que absoluta.",
    unit_notes:
      "<50 normal · 50–250 limítrofe (verificar novamente) · >250 provável inflamação ativa.",
  },
  homocisteina: {
    high_means:
      "Deficiência de B12/folato/B6, insuficiência renal, hipotiroidismo, metotrexato, polimorfismos MTHFR. Grave (>100): homocistinúria (genética – aterosclerose precoce, luxação do cristalino).",
    description:
      "Aminoácido contendo enxofre no metabolismo da metionina. Níveis elevados são um fator de risco CV/trombótico independente. Reflete o estado de B12/folato/B6.",
    low_means: "Geralmente não sinalizado.",
    unit_notes:
      "Referência <15 µmol/L. Trate primeiro repondo as vitaminas B; a suplementação reduz os níveis, mas os ensaios de resultados CV são neutros.",
  },
};

export function localizeAnalyteText(
  id: string,
  field: AnalyteTextField,
  fallback: string | null | undefined,
): string | null {
  if (appearance.resolvedLocale !== "pt-PT") return fallback ?? null;
  return analytesPtPT[id]?.[field] ?? fallback ?? null;
}
