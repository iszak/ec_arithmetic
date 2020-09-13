#[derive(Debug)]
struct Range<T> {
    start: T,
    end: T,
}

#[derive(Debug)]
struct Data<S> {
    range: Range<f32>,
    probability: f32,
    symbol: S,
}

pub fn encode(model: &Model, symbols: Vec<&str>) -> (f32, f32) {
    let mut encode_start = model.start;
    let mut encode_end = model.end;

    for symbol in symbols {
        match model.iterator.iter().find(|d| d.symbol == symbol) {
            Some(data) => {
                let range = &data.range;

                // TODO: Find better name than interval
                let interval = encode_end - encode_start;

                encode_end = encode_start + (interval * range.end);
                encode_start = encode_start + (interval * range.start);
            }
            None => panic!("no symbol found"),
        }
    }

    return (encode_start, encode_end);
}

pub fn decode(model: &Model, start: f32, end: f32) -> Vec<String> {
    let mut decode_start = model.start;
    let mut decode_end = model.end;

    let mut decoded: Vec<String> = Vec::new();

    'outer: loop {
        // TODO: Find better name than interval
        let interval = decode_end - decode_start;

        for data in model.iterator.iter() {
            let range = &data.range;

            let a = decode_start + interval * range.start;
            let b = decode_start + interval * range.end;

            if start >= a && end <= b {
                decoded.push(data.symbol.to_string());
                decode_end = b;
                decode_start = a;
                continue 'outer;
            }
        }

        break 'outer;
    }

    return decoded;
}

#[derive(Debug)]
pub struct Probability<S> {
    probability: f32,
    symbol: S,
}

pub struct Model<'a> {
    start: f32,
    end: f32,
    // TODO: Use iterator
    iterator: Vec<Data<&'a str>>,
}

impl Model<'_> {
    pub fn new<'b>(probabilities: Vec<Probability<&str>>) -> Model {
        let mut cumulative = 0.0;

        // TODO: How to guarantee stability in probability?
        let model = probabilities
            .iter()
            .map(|probability| {
                let start = cumulative;
                let end = cumulative + probability.probability;

                // Check if cumulative is < INFINITY also will fail if NaN
                if !end.is_finite() {
                    panic!("cumulative is overflown");
                }

                cumulative = end;

                if cumulative > 1.0 {
                    panic!("frequency exceeds 1.0");
                }

                return Data {
                    range: Range {
                        start: start.clone(),
                        end: end.clone(),
                    },
                    probability: probability.probability,
                    symbol: probability.symbol,
                };
            })
            .collect();

        Model {
            iterator: model,
            start: 0.0,
            end: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode() {
        let probabilities = vec![
            Probability {
                probability: 0.5,
                symbol: "a",
            },
            Probability {
                probability: 0.25,
                symbol: "b",
            },
            Probability {
                probability: 0.125,
                symbol: "c",
            },
        ];

        let model = Model::new(probabilities);

        assert_eq!((0.0, 0.5), encode(&model, vec!["a"]));
        assert_eq!((0.5, 0.75), encode(&model, vec!["b"]));
        assert_eq!((0.75, 0.875), encode(&model, vec!["c"]));
        assert_eq!((0.0, 0.25), encode(&model, vec!["a", "a"]));
        assert_eq!((0.25, 0.375), encode(&model, vec!["a", "b"]));
        assert_eq!((0.5, 0.625), encode(&model, vec!["b", "a"]));
        assert_eq!((0.8125, 0.84375), encode(&model, vec!["c", "b"]));
        assert_eq!((0.34375, 0.359375), encode(&model, vec!["a", "b", "c"]));
        assert_eq!((0.8125, 0.828125), encode(&model, vec!["c", "b", "a"]));
    }

    #[test]
    fn test_decode() {
        let probabilities = vec![
            Probability {
                probability: 0.5,
                symbol: "a",
            },
            Probability {
                probability: 0.25,
                symbol: "b",
            },
            Probability {
                probability: 0.125,
                symbol: "c",
            },
        ];

        let model = Model::new(probabilities);

        assert_eq!(vec!["a"], decode(&model, 0.0, 0.5));
        assert_eq!(vec!["b"], decode(&model, 0.5, 0.75));
        assert_eq!(vec!["c"], decode(&model, 0.75, 0.875));
        assert_eq!(vec!["a", "a"], decode(&model, 0.0, 0.25));
        assert_eq!(vec!["a", "b"], decode(&model, 0.25, 0.375));
        assert_eq!(vec!["b", "a"], decode(&model, 0.5, 0.625));
        assert_eq!(vec!["c", "b"], decode(&model, 0.8125, 0.84375));
        assert_eq!(vec!["a", "b", "c"], decode(&model, 0.34375, 0.359375));
        assert_eq!(vec!["c", "b", "a"], decode(&model, 0.8125, 0.828125));
    }
}
