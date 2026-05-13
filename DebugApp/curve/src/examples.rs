use debug_ui::{
    curve::{ArcCurve, DebugCurve},
    egui::Pos2,
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct CurveExample {
    pub name: String,
    pub curve: DebugCurve,
}

pub struct LoadedExamples {
    pub examples: Vec<CurveExample>,
    pub error: Option<String>,
}

pub fn load_examples() -> LoadedExamples {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");
    let mut examples = Vec::new();
    let mut error = None;

    match json_files(&dir) {
        Ok(paths) => {
            for path in paths {
                match load_example(&path) {
                    Ok(example) => examples.push(example),
                    Err(message) => {
                        error = Some(format!("{}: {message}", file_stem(&path)));
                    }
                }
            }
        }
        Err(message) => error = Some(message),
    }

    if examples.is_empty() {
        examples.push(CurveExample {
            name: "default".to_owned(),
            curve: DebugCurve::default(),
        });
    }

    LoadedExamples { examples, error }
}

fn json_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let entries = fs::read_dir(dir).map_err(|error| format!("{}: {error}", dir.display()))?;
    let mut paths = Vec::new();

    for entry in entries {
        let path = entry
            .map_err(|error| format!("{}: {error}", dir.display()))?
            .path();

        if path
            .extension()
            .is_some_and(|extension| extension == "json")
        {
            paths.push(path);
        }
    }

    paths.sort();

    Ok(paths)
}

fn load_example(path: &Path) -> Result<CurveExample, String> {
    let source = fs::read_to_string(path).map_err(|error| error.to_string())?;

    Ok(CurveExample {
        name: file_stem(path),
        curve: parse_curve(&source)?,
    })
}

fn file_stem(path: &Path) -> String {
    path.file_stem()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_owned()
}

fn parse_curve(source: &str) -> Result<DebugCurve, String> {
    let value = JsonParser::new(source).parse()?;
    let root = value
        .as_object()
        .ok_or_else(|| "root must be a JSON object".to_owned())?;
    let kind = root
        .field("kind")
        .and_then(JsonValue::as_string)
        .ok_or_else(|| "missing string field `kind`".to_owned())?;

    match kind {
        "line" => Ok(DebugCurve::Line(parse_points(root, 2)?)),
        "quad" => Ok(DebugCurve::Quad(parse_points(root, 3)?)),
        "cubic" => Ok(DebugCurve::Cubic(parse_points(root, 4)?)),
        "arc" => parse_arc(root).map(DebugCurve::Arc),
        _ => Err(format!("unknown curve kind `{kind}`")),
    }
}

fn parse_points<const N: usize>(root: JsonObject<'_>, count: usize) -> Result<[Pos2; N], String> {
    let points = root
        .field("points")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "missing array field `points`".to_owned())?;

    if points.len() != count {
        return Err(format!("`points` must contain {count} points"));
    }

    let mut result = [Pos2::ZERO; N];

    for index in 0..N {
        result[index] = parse_point(&points[index])?;
    }

    Ok(result)
}

fn parse_arc(root: JsonObject<'_>) -> Result<ArcCurve, String> {
    let center = root
        .field("center")
        .map(parse_point)
        .transpose()?
        .ok_or_else(|| "missing point field `center`".to_owned())?;
    let radius = root
        .field("radius")
        .ok_or_else(|| "missing number field `radius`".to_owned())?
        .as_number()?;
    let start_angle = root
        .field("start_deg")
        .ok_or_else(|| "missing number field `start_deg`".to_owned())?
        .as_number()?
        .to_radians();
    let sweep_angle = root
        .field("sweep_deg")
        .ok_or_else(|| "missing number field `sweep_deg`".to_owned())?
        .as_number()?
        .to_radians();

    Ok(ArcCurve {
        center,
        radius,
        start_angle,
        sweep_angle,
    })
}

fn parse_point(value: &JsonValue) -> Result<Pos2, String> {
    let array = value
        .as_array()
        .ok_or_else(|| "point must be an array".to_owned())?;

    if array.len() != 2 {
        return Err("point must contain two numbers".to_owned());
    }

    Ok(Pos2::new(array[0].as_number()?, array[1].as_number()?))
}

enum JsonValue {
    Object(Vec<(String, JsonValue)>),
    Array(Vec<JsonValue>),
    String(String),
    Number(f32),
    Other,
}

impl JsonValue {
    fn as_object(&self) -> Option<JsonObject<'_>> {
        match self {
            Self::Object(fields) => Some(JsonObject { fields }),
            _ => None,
        }
    }

    fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    fn as_string(&self) -> Option<&str> {
        match self {
            Self::String(value) => Some(value),
            _ => None,
        }
    }

    fn as_number(&self) -> Result<f32, String> {
        match self {
            Self::Number(value) => Ok(*value),
            _ => Err("expected number".to_owned()),
        }
    }
}

#[derive(Clone, Copy)]
struct JsonObject<'a> {
    fields: &'a [(String, JsonValue)],
}

impl JsonObject<'_> {
    fn field(&self, key: &str) -> Option<&JsonValue> {
        self.fields
            .iter()
            .find_map(|(name, value)| (name == key).then_some(value))
    }
}

struct JsonParser<'a> {
    source: &'a str,
    index: usize,
}

impl<'a> JsonParser<'a> {
    fn new(source: &'a str) -> Self {
        Self { source, index: 0 }
    }

    fn parse(mut self) -> Result<JsonValue, String> {
        let value = self.parse_value()?;
        self.skip_whitespace();

        if self.index == self.source.len() {
            Ok(value)
        } else {
            Err("unexpected trailing input".to_owned())
        }
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        match self.peek_byte() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(b't') => self.parse_literal("true"),
            Some(b'f') => self.parse_literal("false"),
            Some(b'n') => self.parse_literal("null"),
            Some(byte) => Err(format!("unexpected byte `{}`", byte as char)),
            None => Err("unexpected end of input".to_owned()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'{')?;
        let mut fields = Vec::new();

        loop {
            self.skip_whitespace();

            if self.consume_byte(b'}') {
                break;
            }

            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            let value = self.parse_value()?;
            fields.push((key, value));
            self.skip_whitespace();

            if self.consume_byte(b'}') {
                break;
            }

            self.expect_byte(b',')?;
        }

        Ok(JsonValue::Object(fields))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'[')?;
        let mut values = Vec::new();

        loop {
            self.skip_whitespace();

            if self.consume_byte(b']') {
                break;
            }

            values.push(self.parse_value()?);
            self.skip_whitespace();

            if self.consume_byte(b']') {
                break;
            }

            self.expect_byte(b',')?;
        }

        Ok(JsonValue::Array(values))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"')?;
        let mut result = String::new();

        loop {
            let byte = self
                .next_byte()
                .ok_or_else(|| "unterminated string".to_owned())?;

            match byte {
                b'"' => break,
                b'\\' => {
                    let escaped = self
                        .next_byte()
                        .ok_or_else(|| "unterminated string escape".to_owned())?;
                    match escaped {
                        b'"' => result.push('"'),
                        b'\\' => result.push('\\'),
                        b'/' => result.push('/'),
                        b'b' => result.push('\u{0008}'),
                        b'f' => result.push('\u{000c}'),
                        b'n' => result.push('\n'),
                        b'r' => result.push('\r'),
                        b't' => result.push('\t'),
                        _ => return Err("unsupported string escape".to_owned()),
                    }
                }
                _ => result.push(byte as char),
            }
        }

        Ok(result)
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.index;

        self.consume_byte(b'-');
        self.consume_digits();

        if self.consume_byte(b'.') {
            self.consume_digits();
        }

        if self.consume_byte(b'e') || self.consume_byte(b'E') {
            let _ = self.consume_byte(b'+') || self.consume_byte(b'-');
            self.consume_digits();
        }

        self.source[start..self.index]
            .parse::<f32>()
            .map(JsonValue::Number)
            .map_err(|error| format!("invalid number: {error}"))
    }

    fn parse_literal(&mut self, literal: &str) -> Result<JsonValue, String> {
        if self.source[self.index..].starts_with(literal) {
            self.index += literal.len();
            Ok(JsonValue::Other)
        } else {
            Err(format!("expected `{literal}`"))
        }
    }

    fn consume_digits(&mut self) {
        while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
            self.index += 1;
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.index += 1;
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        if self.consume_byte(expected) {
            Ok(())
        } else {
            Err(format!("expected `{}`", expected as char))
        }
    }

    fn consume_byte(&mut self, expected: u8) -> bool {
        if self.peek_byte() == Some(expected) {
            self.index += 1;
            true
        } else {
            false
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.index += 1;
        Some(byte)
    }

    fn peek_byte(&self) -> Option<u8> {
        self.source.as_bytes().get(self.index).copied()
    }
}
