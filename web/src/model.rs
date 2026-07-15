//! Data model and seed data for the Neoland web console.

#[derive(Clone)]
pub struct Session {
    pub title: &'static str,
    pub time: &'static str,
    pub state: &'static str,
}

#[derive(Clone)]
pub struct Message {
    pub role: &'static str,
    pub body: String,
    pub code: Option<&'static str>,
}

pub const SESSIONS: [Session; 4] = [
    Session {
        title: "Refatorar módulo de auth",
        time: "agora",
        state: "live",
    },
    Session {
        title: "Revisar pipeline de deploy",
        time: "18m",
        state: "hot",
    },
    Session {
        title: "Otimizar query de eventos",
        time: "2h",
        state: "hot",
    },
    Session {
        title: "Brainstorm de arquitetura",
        time: "1d",
        state: "idle",
    },
];

pub fn seed_messages() -> Vec<Message> {
    vec![
        Message {
            role: "agent",
            body: "Analisei o hot path. O gargalo está no executor síncrono: cada unidade espera a anterior liberar o GIL.".into(),
            code: Some("from multiprocessing import Pool\n\ndef run(items):\n    with Pool() as pool:\n        return pool.map(work, items)"),
        },
        Message {
            role: "user",
            body: "Otimiza esse script Python pra performance e preserva a ordem dos resultados.".into(),
            code: None,
        },
        Message {
            role: "agent",
            body: "Sugiro um pool de processos com map ordenado. Isso distribui o trabalho entre os cores sem mudar o contrato de saída.".into(),
            code: None,
        },
    ]
}
