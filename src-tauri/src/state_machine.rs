//! The single source of truth for the recording lifecycle.
//! The UI never infers state; it renders whatever the engine reports.
use serde::Serialize;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecState {
    Idle,
    Preparing,
    Recording,
    Paused,
    Stopping,
    Error,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecEvent {
    Start,
    Started,
    Pause,
    Resume,
    Stop,
    Stopped,
    Fail,
    Reset,
}

#[derive(Debug, PartialEq, Eq)]
pub struct InvalidTransition(pub RecState, pub RecEvent);

pub fn next(state: RecState, event: RecEvent) -> Result<RecState, InvalidTransition> {
    use RecEvent as E;
    use RecState as S;
    Ok(match (state, event) {
        (S::Idle | S::Error, E::Start) => S::Preparing,
        (S::Preparing, E::Started) => S::Recording,
        (S::Recording, E::Pause) => S::Paused,
        (S::Paused, E::Resume) => S::Recording,
        (S::Recording | S::Paused, E::Stop) => S::Stopping,
        (S::Preparing, E::Stop) => S::Idle,
        (S::Stopping, E::Stopped) => S::Idle,
        (S::Preparing | S::Recording | S::Paused | S::Stopping, E::Fail) => S::Error,
        (S::Error, E::Reset) => S::Idle,
        (s, e) => return Err(InvalidTransition(s, e)),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use RecEvent as E;
    use RecState as S;

    #[test]
    fn happy_path() {
        let mut s = S::Idle;
        for (e, want) in [
            (E::Start, S::Preparing),
            (E::Started, S::Recording),
            (E::Pause, S::Paused),
            (E::Resume, S::Recording),
            (E::Stop, S::Stopping),
            (E::Stopped, S::Idle),
        ] {
            s = next(s, e).unwrap();
            assert_eq!(s, want);
        }
    }

    #[test]
    fn cannot_double_start_or_stop_when_idle() {
        assert!(next(S::Recording, E::Start).is_err());
        assert!(next(S::Idle, E::Stop).is_err());
        assert!(next(S::Idle, E::Pause).is_err());
    }

    #[test]
    fn errors_are_recoverable() {
        assert_eq!(next(S::Recording, E::Fail), Ok(S::Error));
        assert_eq!(next(S::Error, E::Start), Ok(S::Preparing));
        assert_eq!(next(S::Error, E::Reset), Ok(S::Idle));
    }

    #[test]
    fn cancel_while_preparing_returns_to_idle() {
        assert_eq!(next(S::Preparing, E::Stop), Ok(S::Idle));
    }
}
