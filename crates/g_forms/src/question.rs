//! Implementations for the [Question] data type.
#![allow(unused)]

use std::{fmt::Debug, marker::PhantomData, str::FromStr};

use chrono::{NaiveDateTime, NaiveTime};
use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Number;

use crate::{
    form::{FormResponse, InputValidation, QuestionType, SelectionLimits, SingleSelection},
    raw::{DateType, RawInputValidation, RawQuestionInfo, TimeType},
};

use self::question_types::{
    CheckBox, Date, DropDown, Grid, LinearScale, LongAnswer, MultipleChoice, ShortAnswer, Time,
};

lazy_static! {
    /// Email matching regex
    static ref REGEX_EMAIL: Regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();

}
/// Unit types that implement this trait can be used
/// as a question type.
pub trait IsQuestion {}

pub mod question_types {
    use super::IsQuestion;

    #[derive(Clone, Debug, Default)]
    pub struct ShortAnswer {}
    #[derive(Clone, Debug, Default)]
    pub struct LongAnswer {}
    #[derive(Clone, Debug, Default)]
    pub struct MultipleChoice {}
    #[derive(Clone, Debug, Default)]
    pub struct DropDown {}
    #[derive(Clone, Debug, Default)]
    pub struct CheckBox {}
    #[derive(Clone, Debug, Default)]
    pub struct LinearScale {}
    #[derive(Clone, Debug, Default)]
    pub struct Grid {}
    #[derive(Clone, Debug, Default)]
    pub struct Date {}
    #[derive(Clone, Debug, Default)]
    pub struct Time {}

    impl IsQuestion for ShortAnswer {}
    impl IsQuestion for LongAnswer {}
    impl IsQuestion for MultipleChoice {}
    impl IsQuestion for DropDown {}
    impl IsQuestion for CheckBox {}
    impl IsQuestion for LinearScale {}
    impl IsQuestion for Grid {}
    impl IsQuestion for Date {}
    impl IsQuestion for Time {}
}

/// A google form question.
/// Question types are discriminated using an unit state struct.
#[derive(Clone, Debug, Default)]
pub struct Question<T: Clone + Debug + Default + IsQuestion> {
    // represents the actual question stored
    marker: PhantomData<T>,

    /// Form response, always in string form.
    ///
    /// Same for all question types.
    response: Option<String>,

    // the following fields are valid for open ended type questions:
    // - short answer
    // - long answer
    /// Input validation, for some question types.
    input_validation: Option<InputValidation>,

    /// Error message for when input validation fails
    validation_error: Option<String>,

    // the following are valid for selection type questions:
    // - multiple choice (choose only one)
    // - check box (choose one or more)
    // - drop down (variant of multiple choice)
    /// Vector of available choices
    inner: Option<Vec<SingleSelection>>,

    // /// Multi-select?
    // multiple: bool,
    /// Lower and upper limits
    /// for linear scale questions
    limits: Option<SelectionLimits>,

    // fields for date-time questions
    date_type: Option<DateType>,

    time_type: Option<TimeType>,

    date_time: Option<chrono::NaiveDateTime>,
}

/// inner try from for short answer and long answer
fn raw_to_open_ended<T: Clone + Debug + Default + IsQuestion>(
    val: Vec<RawQuestionInfo>,
) -> Result<Question<T>, ()> {
    // vec should have a valid first element
    // for open ended questions
    let inner = val.into_iter().next().ok_or(())?;

    let mut qn = Question::default();

    let raw: Option<RawInputValidation> = inner
        .input_validation
        .and_then(|v| v.iter().next().and_then(|elem| Some(elem.to_owned())));

    qn.validation_error = raw.as_ref().and_then(|v| v.error_text.clone());
    qn.input_validation = raw.and_then(|v| Some(InputValidation::try_from(v).ok()?));

    Ok(qn)
}

/// inner try from for MCQ, Drop down and checkbox
fn raw_to_selection<T: Clone + Debug + Default + IsQuestion>(
    val: Vec<RawQuestionInfo>,
) -> Result<Question<T>, ()> {
    let raw = val.into_iter().next().ok_or(())?;

    let options = raw
        .dimension_1
        .ok_or(())?
        .into_iter()
        .map(|elem| SingleSelection {
            answer: elem.name,
            selected: false,
        })
        .collect::<Vec<SingleSelection>>();

    let limits: Option<SelectionLimits> = match raw.dimension_2 {
        Some(_limit) => {
            let chunk = _limit.chunks(2).next().ok_or(())?;
            Some(SelectionLimits {
                lower: chunk[0].clone(),
                upper: chunk[1].clone(),
            })
        }
        None => None,
    };

    let validation: Option<InputValidation> = match raw.input_validation {
        Some(_iv) => {
            let inner = _iv.into_iter().next().ok_or(())?;
            Some(InputValidation::try_from(inner)?)
        }
        None => None,
    };

    let mut qn = Question::default();
    qn.inner = Some(options);
    qn.limits = limits;
    qn.input_validation = validation;

    Ok(qn)
}

/// inner try from for date questions
fn raw_to_date<T: Clone + Debug + Default + IsQuestion>(
    val: Vec<RawQuestionInfo>,
) -> Result<Question<T>, ()> {
    let raw = val.into_iter().next().ok_or(())?;
    let date = DateType::try_from(raw.date_type.ok_or(())?)?;

    let mut qn = Question::default();
    qn.date_type = Some(date);

    Ok(qn)
}
/// inner try from for time questions
fn raw_to_time<T: Clone + Debug + Default + IsQuestion>(
    val: Vec<RawQuestionInfo>,
) -> Result<Question<T>, ()> {
    let raw = val.into_iter().next().ok_or(())?;

    let mut qn = Question::default();

    qn.time_type = Some(raw.time_type.ok_or(())?.inner);

    Ok(qn)
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<ShortAnswer> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_open_ended(value)
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<LongAnswer> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_open_ended(value)
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<MultipleChoice> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_selection(value)
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<DropDown> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_selection(value)
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<CheckBox> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_selection(value)
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<LinearScale> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_selection(value)
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<Grid> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        todo!()
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<Date> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_date(value)
    }
}

impl TryFrom<Vec<RawQuestionInfo>> for Question<Time> {
    type Error = ();

    fn try_from(value: Vec<RawQuestionInfo>) -> Result<Self, Self::Error> {
        raw_to_time(value)
    }
}

/// Return the stringified response for a particular question
impl<T: Clone + Debug + Default + IsQuestion> FormResponse for Question<T> {
    fn form_response(&self) -> Option<String> {
        self.response.clone()
    }
}

/// Alias for a unit result type
pub type FillResult = Result<(), ()>;

// private implementations here
impl<T: Clone + Debug + Default + IsQuestion> Question<T> {
    // /// Pushes the question-response pair to an internal map.
    // /// If the key exists, its value is updated.
    // fn _add_to_map(&mut self, id: u64, resp: String) {
    //     let key = format!("entry.{}", id);

    // }

    fn _fill_number(&mut self, resp: Number) -> FillResult {
        // input validation
        if let Some(validation) = &self.input_validation {
            match validation {
                InputValidation::NumberGT(num) => {
                    if !(resp.as_f64().ok_or(())? > num.as_f64().ok_or(())?) {
                        return Err(());
                    }
                }
                InputValidation::NumberGTE(num) => {
                    if !(resp.as_f64().ok_or(())? >= num.as_f64().ok_or(())?) {
                        return Err(());
                    }
                }
                InputValidation::NumberLT(num) => {
                    if !(resp.as_f64().ok_or(())? < num.as_f64().ok_or(())?) {
                        return Err(());
                    }
                }
                InputValidation::NumberLTE(num) => {
                    if !(resp.as_f64().ok_or(())? <= num.as_f64().ok_or(())?) {
                        return Err(());
                    }
                }
                InputValidation::NumberEQ(num) => {
                    if !(&resp == num) {
                        return Err(());
                    }
                }
                InputValidation::NumberNEQ(num) => {
                    if !(&resp != num) {
                        return Err(());
                    }
                }
                InputValidation::NumberBT(num_a, num_b) => {
                    let a = num_a.as_f64();
                    let b = num_b.as_f64();
                    let comp = resp.as_f64();

                    if !(a <= comp && comp <= b) {
                        return Err(());
                    }
                }
                InputValidation::NumberNBT(num_a, num_b) => {
                    let a = num_a.as_f64();
                    let b = num_b.as_f64();
                    let comp = resp.as_f64();

                    if !(comp < a && b < comp) {
                        return Err(());
                    }
                }
                InputValidation::NumberIsNumber => (),
                InputValidation::NumberIsWhole => {
                    if !resp.is_i64() {
                        return Err(());
                    }
                }

                _ => return Err(()),
            }
        }

        self.response = Some(resp.to_string());

        Ok(())
    }

    /// For open-ended type questions:
    /// - short answer
    /// - long answer
    ///
    /// If the response does not pass input validation,
    /// it will return an error.
    fn _fill_str(&mut self, resp: &str) -> FillResult {
        if let Some(validation) = &self.input_validation {
            match validation {
                InputValidation::NumberGT(_)
                | InputValidation::NumberGTE(_)
                | InputValidation::NumberLT(_)
                | InputValidation::NumberLTE(_)
                | InputValidation::NumberEQ(_)
                | InputValidation::NumberNEQ(_)
                | InputValidation::NumberBT(_, _)
                | InputValidation::NumberNBT(_, _)
                | InputValidation::NumberIsNumber
                | InputValidation::NumberIsWhole => {
                    let num = Number::from_str(resp).ok().ok_or(())?;
                    return self._fill_number(num);
                }

                InputValidation::TextContains(text) => {
                    if !resp.contains(text) {
                        return Err(());
                    }
                }
                InputValidation::TextNotContains(text) => {
                    if resp.contains(text) {
                        return Err(());
                    }
                }
                InputValidation::TextIsUrl => match reqwest::Url::try_from(resp) {
                    Ok(_) => (),
                    Err(_) => return Err(()),
                },
                InputValidation::TextIsEmail => match REGEX_EMAIL.is_match(resp) {
                    true => (),
                    false => return Err(()),
                },
                // InputValidation::CheckBoxGTE(_) => todo!(),
                // InputValidation::CheckBoxLTE(_) => todo!(),
                // InputValidation::CheckBoxEQ(_) => todo!(),
                InputValidation::LengthMaximumChars(len) => {
                    if !(resp.len() <= *len as usize) {
                        return Err(());
                    }
                }
                InputValidation::LengthMinimumChars(len) => {
                    if !(resp.len() >= *len as usize) {
                        return Err(());
                    }
                }
                InputValidation::RegexContains(re) => todo!(),
                InputValidation::RegexNotContains(re) => todo!(),
                InputValidation::RegexMatches(re) => todo!(),
                InputValidation::RegexNotMatches(re) => todo!(),

                _ => return Err(()),
            }
        }

        self.response = Some(resp.to_owned());

        Ok(())
    }

    /// For selection questions.
    /// Fill the response from the numbered option.
    fn _fill_option(&mut self, resp: usize) -> FillResult {
        let opt = self.inner.as_mut().ok_or(())?.get_mut(resp).ok_or(())?;
        opt.selected = true;
        self.response = Some(opt.answer.to_owned());

        Ok(())
    }

    fn _fill_date(&mut self, resp: NaiveDateTime) -> FillResult {
        self.date_time = Some(resp);

        // this took so long to get right AHHHHHHH
        match self.date_type.ok_or(())? {
            DateType::Date => {
                let resp_str = resp.format("%Y-%m-%d").to_string();
                self.response = Some(resp_str);
            }
            DateType::DateYear => {
                let resp_str = resp.format("%Y-%m-%d").to_string();
                self.response = Some(resp_str);
            }
            DateType::DateTime => {
                let resp_str = resp.format("%Y-%m-%d %H:%M:00").to_string();
                self.response = Some(resp_str);
            }
            DateType::DateTimeYear => {
                let resp_str = resp.format("%Y-%m-%d %H:%M:00").to_string();
                self.response = Some(resp_str);
            }
        }

        Ok(())
    }

    fn _fill_time(&mut self, resp: NaiveTime) -> FillResult {
        let date_part = chrono::Local::now().date_naive();
        let combined = NaiveDateTime::new(date_part, resp);
        self.date_time = Some(combined);

        match self.time_type.ok_or(())? {
            TimeType::Time => {
                let resp_str = resp.format("%H:%M:00").to_string();
                self.response = Some(resp_str);
            }

            // valid for durations up to 23H 59M 59S. (piggybacking on time type)
            TimeType::Duration => {
                let resp_str = resp.format("%H:%M:%S").to_string();
                self.response = Some(resp_str);
            }
        }

        Ok(())
    }
}

impl Question<ShortAnswer> {
    pub fn fill_str(&mut self, resp: &str) -> FillResult {
        self._fill_str(resp)
    }

    pub fn fill_number(&mut self, resp: Number) -> FillResult {
        self._fill_number(resp)
    }
}

impl Question<LongAnswer> {
    pub fn fill_str(&mut self, resp: &str) -> FillResult {
        self._fill_str(resp)
    }
}

impl Question<MultipleChoice> {
    pub fn fill_option(&mut self, resp: usize) -> FillResult {
        self._fill_option(resp)
    }
}

impl Question<DropDown> {
    pub fn fill_option(&mut self, resp: usize) -> FillResult {
        self._fill_option(resp)
    }
}

impl Question<CheckBox> {
    pub fn fill_option(&mut self, resp: usize) -> FillResult {
        self._fill_option(resp)
    }
}

impl Question<LinearScale> {
    pub fn fill_option(&mut self, resp: usize) -> FillResult {
        self._fill_option(resp)
    }
}

impl Question<Grid> {}

impl Question<Date> {
    pub fn fill_date(&mut self, resp: NaiveDateTime) -> FillResult {
        self._fill_date(resp)
    }
}

impl Question<Time> {
    pub fn fill_time(&mut self, resp: NaiveTime) -> FillResult {
        self._fill_time(resp)
    }
}

/// Use these methods if the question type is known
impl QuestionType {
    pub fn fill_str(&mut self, resp: &str) -> FillResult {
        match self {
            QuestionType::ShortAnswer(qn) => qn._fill_str(resp),
            QuestionType::LongAnswer(qn) => qn._fill_str(resp),
            QuestionType::MultipleChoice(_) => Err(()),
            QuestionType::DropDown(_) => Err(()),
            QuestionType::CheckBox(_) => Err(()),
            QuestionType::LinearScale(_) => Err(()),
            QuestionType::Grid => Err(()),
            QuestionType::Date(_) => Err(()),
            QuestionType::Time(_) => Err(()),
        }
    }

    pub fn fill_number(&mut self, resp: Number) -> FillResult {
        match self {
            QuestionType::ShortAnswer(qn) => qn._fill_number(resp),
            QuestionType::LongAnswer(qn) => qn._fill_number(resp),
            QuestionType::MultipleChoice(_) => Err(()),
            QuestionType::DropDown(_) => Err(()),
            QuestionType::CheckBox(_) => Err(()),
            QuestionType::LinearScale(_) => Err(()),
            QuestionType::Grid => Err(()),
            QuestionType::Date(_) => Err(()),
            QuestionType::Time(_) => Err(()),
        }
    }

    pub fn fill_option(&mut self, resp: usize) -> FillResult {
        match self {
            QuestionType::ShortAnswer(_) => Err(()),
            QuestionType::LongAnswer(_) => Err(()),
            QuestionType::MultipleChoice(qn) => qn._fill_option(resp),
            QuestionType::DropDown(qn) => qn._fill_option(resp),
            QuestionType::CheckBox(qn) => qn._fill_option(resp),
            QuestionType::LinearScale(qn) => qn._fill_option(resp),
            QuestionType::Grid => Err(()),
            QuestionType::Date(_) => Err(()),
            QuestionType::Time(_) => Err(()),
        }
    }
    pub fn fill_date(&mut self, resp: NaiveDateTime) -> FillResult {
        match self {
            QuestionType::ShortAnswer(_) => Err(()),
            QuestionType::LongAnswer(_) => Err(()),
            QuestionType::MultipleChoice(_) => Err(()),
            QuestionType::DropDown(_) => Err(()),
            QuestionType::CheckBox(_) => Err(()),
            QuestionType::LinearScale(_) => Err(()),
            QuestionType::Grid => Err(()),
            QuestionType::Date(qn) => qn._fill_date(resp),
            QuestionType::Time(_) => Err(()),
        }
    }

    pub fn fill_time(&mut self, resp: NaiveTime) -> FillResult {
        match self {
            QuestionType::ShortAnswer(_) => Err(()),
            QuestionType::LongAnswer(_) => Err(()),
            QuestionType::MultipleChoice(_) => Err(()),
            QuestionType::DropDown(_) => Err(()),
            QuestionType::CheckBox(_) => Err(()),
            QuestionType::LinearScale(_) => Err(()),
            QuestionType::Grid => Err(()),
            QuestionType::Date(_) => Err(()),
            QuestionType::Time(qn) => qn._fill_time(resp),
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use chrono::NaiveDateTime;
    use chrono::NaiveTime;
    use serde_json::Number;

    use crate::form::FormResponse;

    use super::question_types::*;
    use super::Question;

    /// Check that stringified answers conforms to google's spec
    #[test]
    fn test_string_answers() {
        let mut qn_short = Question::<ShortAnswer>::default();
        qn_short.fill_number(Number::from(100)).unwrap();
        let stringified = qn_short.form_response().unwrap();
        assert_eq!(stringified, "100");

        qn_short
            .fill_number(Number::from_f64(3.14).unwrap())
            .unwrap();
        let stringified = qn_short.form_response().unwrap();
        assert_eq!(stringified, "3.14");

        let mut qn_date = Question::<Date>::default();
        let d = NaiveDate::from_ymd_opt(1970, 12, 24).unwrap();
        let t = NaiveTime::from_hms_opt(9, 18, 27).unwrap();
        let dt = NaiveDateTime::new(d, t);

        qn_date.date_type = Some(crate::raw::DateType::Date);
        qn_date.fill_date(dt).unwrap();
        let stringified = qn_date.form_response().unwrap();
        assert_eq!(stringified, "24/12");

        qn_date.date_type = Some(crate::raw::DateType::DateTime);
        qn_date.fill_date(dt).unwrap();
        let stringified = qn_date.form_response().unwrap();
        assert_eq!(stringified, "24/12 09:18:00");

        qn_date.date_type = Some(crate::raw::DateType::DateTimeYear);
        qn_date.fill_date(dt).unwrap();
        let stringified = qn_date.form_response().unwrap();
        assert_eq!(stringified, "24/12/1970 09:18:00");

        qn_date.date_type = Some(crate::raw::DateType::DateYear);
        qn_date.fill_date(dt).unwrap();
        let stringified = qn_date.form_response().unwrap();
        assert_eq!(stringified, "24/12/1970");

        let mut qn_time = Question::<Time>::default();
        qn_time.time_type = Some(crate::raw::TimeType::Time);
        qn_time.fill_time(t).unwrap();
        let stringified = qn_time.form_response().unwrap();
        assert_eq!(stringified, "09:18:00");

        qn_time.time_type = Some(crate::raw::TimeType::Duration);
        qn_time.fill_time(t).unwrap();
        let stringified = qn_time.form_response().unwrap();
        assert_eq!(stringified, "09:18:27");
    }

    #[test]
    fn asd() {
        let x = reqwest::Url::try_from("http://asd.com");
        assert!(matches!(x, Err(_)))
    }
}
