use std::convert::Infallible;
use std::fmt;

// pub(crate) type Selected<T> = Core<route_recognizer::Router<T>>;
pub(crate) type Selected<T> = Core<matchit::Node<T>>;

pub(crate) trait CoreRouter: Clone {
    type Value;
    type AddError: std::error::Error + Send + Sync + 'static;
    type MatchError: std::error::Error + Send + Sync + 'static;

    fn new() -> Self;

    fn add(&mut self, path: &str, value: Self::Value) -> Result<(), Self::AddError>;

    fn route(&self, path: &str) -> Result<Match<&Self::Value>, Self::MatchError>;
}

impl<T: Clone> CoreRouter for route_recognizer::Router<T> {
    type Value = T;
    type AddError = Infallible;
    type MatchError = RouteError;

    fn new() -> Self {
        route_recognizer::Router::new()
    }

    fn add(&mut self, path: &str, value: Self::Value) -> Result<(), Self::AddError> {
        self.add(path, value);
        Ok(())
    }

    fn route(&self, path: &str) -> Result<Match<&Self::Value>, Self::MatchError> {
        let matched = self.recognize(path).map_err(RouteError::new)?;

        let _params = matched
            .params()
            .iter()
            .map(|(s1, s2)| (s1.to_owned(), s2.to_owned()))
            .collect();

        let value = *matched.handler();

        let matched = Match { value, _params };

        Ok(matched)
    }
}

impl<T: Clone> CoreRouter for matchit::Node<T> {
    type Value = T;
    type AddError = matchit::InsertError;
    type MatchError = matchit::MatchError;

    fn new() -> Self {
        matchit::Node::new()
    }

    fn add(&mut self, path: &str, value: Self::Value) -> Result<(), Self::AddError> {
        self.insert(path, value)
    }

    fn route(&self, path: &str) -> Result<Match<&Self::Value>, Self::MatchError> {
        let matched = self.at(path)?;

        let _params = matched
            .params
            .iter()
            .map(|(s1, s2)| (s1.to_owned(), s2.to_owned()))
            .collect();

        let value = matched.value;

        let matched = Match { value, _params };

        Ok(matched)
    }
}

#[derive(Clone)]
pub(crate) struct Core<T> {
    router: T,
}

impl<T: CoreRouter> Core<T> {
    pub(crate) fn new() -> Self {
        Self { router: T::new() }
    }

    pub(crate) fn add(&mut self, path: &str, value: T::Value) -> Result<(), T::AddError> {
        self.router.add(path, value)
    }

    pub(crate) fn route(&self, path: &str) -> Result<Match<&T::Value>, T::MatchError> {
        self.router.route(path)
    }
}

pub(crate) struct Match<T> {
    pub value: T,
    pub _params: Vec<(String, String)>,
}

#[derive(Debug)]
pub(crate) struct RouteError {
    message: String,
}

impl RouteError {
    fn new(message: String) -> Self {
        Self { message }
    }
}

impl fmt::Display for RouteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RouteError {}
