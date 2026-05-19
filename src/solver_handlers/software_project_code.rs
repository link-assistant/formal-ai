pub(super) struct CodeArtifact {
    pub(super) label: &'static str,
    pub(super) fence: &'static str,
    pub(super) body: &'static str,
}

pub(super) fn implementation_code(
    game_tracker: bool,
    implementation_language: &str,
) -> CodeArtifact {
    if game_tracker {
        return CodeArtifact {
            label: "TypeScript",
            fence: "typescript",
            body: GAME_TRACKER_TYPESCRIPT,
        };
    }
    match implementation_language {
        "python" => CodeArtifact {
            label: "Python",
            fence: "python",
            body: GENERIC_PROJECT_PYTHON,
        },
        "rust" => CodeArtifact {
            label: "Rust",
            fence: "rust",
            body: GENERIC_PROJECT_RUST,
        },
        "javascript" => CodeArtifact {
            label: "JavaScript",
            fence: "javascript",
            body: GENERIC_PROJECT_JAVASCRIPT,
        },
        _ => CodeArtifact {
            label: "TypeScript",
            fence: "typescript",
            body: GENERIC_PROJECT_TYPESCRIPT,
        },
    }
}

const GAME_TRACKER_TYPESCRIPT: &str = r"type Cooldown = {
  name: string;
  remainingRounds: number;
};

type UnitState = {
  id: string;
  name: string;
  hp: number;
  maxHp: number;
  protection: number;
  resistance: number;
  cooldowns: Cooldown[];
};

type DamageResult = {
  damageTaken: number;
  prevented: number;
  unit: UnitState;
};

export function mitigateDamage(unit: UnitState, rawDamage: number): DamageResult {
  const prevented = Math.max(0, unit.protection) + Math.max(0, unit.resistance);
  const damageTaken = Math.max(0, rawDamage - prevented);
  return {
    damageTaken,
    prevented,
    unit: { ...unit, hp: Math.max(0, unit.hp - damageTaken) },
  };
}

export function setStacks(
  unit: UnitState,
  protection: number,
  resistance: number,
): UnitState {
  return {
    ...unit,
    protection: Math.max(0, protection),
    resistance: Math.max(0, resistance),
  };
}

export function tickCooldowns(unit: UnitState): UnitState {
  return {
    ...unit,
    cooldowns: unit.cooldowns
      .map((cooldown) => ({
        ...cooldown,
        remainingRounds: Math.max(0, cooldown.remainingRounds - 1),
      }))
      .filter((cooldown) => cooldown.remainingRounds > 0),
  };
}";

const GENERIC_PROJECT_TYPESCRIPT: &str = r#"type ProjectRecord = {
  id: string;
  title: string;
  status: "open" | "done";
  notes: string[];
};

type ProjectCommand =
  | { type: "add"; id: string; title: string }
  | { type: "note"; id: string; note: string }
  | { type: "complete"; id: string };

export function applyCommand(
  records: ProjectRecord[],
  command: ProjectCommand,
): ProjectRecord[] {
  switch (command.type) {
    case "add":
      return [
        ...records,
        { id: command.id, title: command.title, status: "open", notes: [] },
      ];
    case "note":
      return records.map((record) =>
        record.id === command.id
          ? { ...record, notes: [...record.notes, command.note] }
          : record,
      );
    case "complete":
      return records.map((record) =>
        record.id === command.id ? { ...record, status: "done" } : record,
      );
  }
}"#;

const GENERIC_PROJECT_JAVASCRIPT: &str = r#"export function applyCommand(records, command) {
  switch (command.type) {
    case "add":
      return [...records, { id: command.id, title: command.title, status: "open", notes: [] }];
    case "note":
      return records.map((record) =>
        record.id === command.id
          ? { ...record, notes: [...record.notes, command.note] }
          : record,
      );
    case "complete":
      return records.map((record) =>
        record.id === command.id ? { ...record, status: "done" } : record,
      );
    default:
      throw new Error(`Unknown command: ${command.type}`);
  }
}"#;

const GENERIC_PROJECT_PYTHON: &str = r#"from dataclasses import dataclass, field


@dataclass(frozen=True)
class ProjectRecord:
    id: str
    title: str
    status: str = "open"
    notes: tuple[str, ...] = field(default_factory=tuple)


def apply_command(records: tuple[ProjectRecord, ...], command: dict) -> tuple[ProjectRecord, ...]:
    if command["type"] == "add":
        return (*records, ProjectRecord(id=command["id"], title=command["title"]))
    if command["type"] == "note":
        return tuple(
            ProjectRecord(r.id, r.title, r.status, (*r.notes, command["note"]))
            if r.id == command["id"] else r
            for r in records
        )
    if command["type"] == "complete":
        return tuple(
            ProjectRecord(r.id, r.title, "done", r.notes)
            if r.id == command["id"] else r
            for r in records
        )
    raise ValueError(f"Unknown command: {command['type']}")
"#;

const GENERIC_PROJECT_RUST: &str = r"#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRecord {
    pub id: String,
    pub title: String,
    pub status: ProjectStatus,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectStatus {
    Open,
    Done,
}

pub enum ProjectCommand {
    Add { id: String, title: String },
    Note { id: String, note: String },
    Complete { id: String },
}

pub fn apply_command(mut records: Vec<ProjectRecord>, command: ProjectCommand) -> Vec<ProjectRecord> {
    match command {
        ProjectCommand::Add { id, title } => records.push(ProjectRecord {
            id,
            title,
            status: ProjectStatus::Open,
            notes: Vec::new(),
        }),
        ProjectCommand::Note { id, note } => {
            for record in &mut records {
                if record.id == id {
                    record.notes.push(note.clone());
                }
            }
        }
        ProjectCommand::Complete { id } => {
            for record in &mut records {
                if record.id == id {
                    record.status = ProjectStatus::Done;
                }
            }
        }
    }
    records
}";
