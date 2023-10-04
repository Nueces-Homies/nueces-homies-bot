use std::fmt::Display;

pub struct UnspecifiedOption;
impl Display for UnspecifiedOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

pub struct AllFields;
pub struct Fields(Vec<String>);

impl Display for AllFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fields *; ")
    }
}

impl Display for Fields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let joined = self.0.join(", ");
        write!(f, "fields {}; ", joined)
    }
}

pub struct WhereClause(String);
impl WhereClause {
    fn new(clause: &str) -> Self {
        WhereClause(format!("where {}", clause))
    }
}

impl Display for WhereClause {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}; ", self.0)
    }
}

pub struct Limit(String);
impl Limit {
    fn new(limit: u32) -> Self {
        Limit(format!("limit {}; ", limit))
    }
}
impl Display for Limit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Offset(String);
impl Offset {
    fn new(offset: u32) -> Self {
        Offset(format!("offset {}; ", offset))
    }
}
impl Display for Offset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Search(String);
impl Search {
    fn new(query: &str) -> Self {
        Search(format!("search \"{}\"; ", query))
    }
}
impl Display for Search {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub enum SortDirection {
    Ascending,
    Descending,
}
pub struct Sort(String, SortDirection);

impl Display for Sort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let direction = match self.1 {
            SortDirection::Ascending => "asc",
            SortDirection::Descending => "desc",
        };

        write!(f, "sort {} {}; ", self.0, direction)
    }
}

pub struct Query(String);

type U = UnspecifiedOption;

impl Query {
    pub fn select(field: &str) -> QueryBuilder<Fields, U, U, U, U, U> {
        let fields = [field.to_owned()].to_vec();

        QueryBuilder {
            fields: Fields(fields),
            where_clause: UnspecifiedOption,
            limit: UnspecifiedOption,
            offset: UnspecifiedOption,
            search: UnspecifiedOption,
            sort: UnspecifiedOption,
        }
    }

    pub fn select_multiple(fields: &[String]) -> QueryBuilder<Fields, U, U, U, U, U> {
        QueryBuilder {
            fields: Fields(fields.to_vec()),
            where_clause: UnspecifiedOption,
            limit: UnspecifiedOption,
            offset: UnspecifiedOption,
            search: UnspecifiedOption,
            sort: UnspecifiedOption,
        }
    }

    pub fn where_(clause: &str) -> QueryBuilder<AllFields, WhereClause, U, U, U, U> {
        QueryBuilder {
            fields: AllFields,
            where_clause: WhereClause::new(clause),
            limit: UnspecifiedOption,
            offset: UnspecifiedOption,
            search: UnspecifiedOption,
            sort: UnspecifiedOption,
        }
    }

    pub fn search(query: &str) -> QueryBuilder<AllFields, U, U, U, Search, U> {
        QueryBuilder {
            fields: AllFields,
            where_clause: UnspecifiedOption,
            limit: UnspecifiedOption,
            offset: UnspecifiedOption,
            search: Search::new(query),
            sort: UnspecifiedOption,
        }
    }

    pub fn limit(limit: u32) -> QueryBuilder<AllFields, U, Limit, U, U, U> {
        QueryBuilder {
            fields: AllFields,
            where_clause: UnspecifiedOption,
            limit: Limit::new(limit),
            offset: UnspecifiedOption,
            search: UnspecifiedOption,
            sort: UnspecifiedOption,
        }
    }

    pub fn offset(offset: u32) -> QueryBuilder<AllFields, U, U, Offset, U, U> {
        QueryBuilder {
            fields: AllFields,
            where_clause: UnspecifiedOption,
            limit: UnspecifiedOption,
            offset: Offset::new(offset),
            search: UnspecifiedOption,
            sort: UnspecifiedOption,
        }
    }

    pub fn sort(
        field: &str,
        direction: SortDirection,
    ) -> QueryBuilder<AllFields, U, U, U, U, Sort> {
        QueryBuilder {
            fields: AllFields,
            where_clause: UnspecifiedOption,
            limit: UnspecifiedOption,
            offset: UnspecifiedOption,
            search: UnspecifiedOption,
            sort: Sort(field.to_owned(), direction),
        }
    }
}

pub struct QueryBuilder<F, WC, L, O, Se, So> {
    fields: F,
    where_clause: WC,
    limit: L,
    offset: O,
    search: Se,
    sort: So,
}

impl<WC, L, O, Se, So> QueryBuilder<AllFields, WC, L, O, Se, So> {
    pub fn select(self, field: &str) -> QueryBuilder<Fields, WC, L, O, Se, So> {
        let fields = [field.to_owned()];

        QueryBuilder {
            fields: Fields(fields.to_vec()),
            where_clause: self.where_clause,
            limit: self.limit,
            offset: self.offset,
            search: self.search,
            sort: self.sort,
        }
    }

    pub fn select_multiple(self, fields: &[String]) -> QueryBuilder<Fields, WC, L, O, Se, So> {
        QueryBuilder {
            fields: Fields(fields.to_vec()),
            where_clause: self.where_clause,
            limit: self.limit,
            offset: self.offset,
            search: self.search,
            sort: self.sort,
        }
    }
}

impl<WC, L, O, Se, So> QueryBuilder<Fields, WC, L, O, Se, So> {
    pub fn select(mut self, field: &str) -> QueryBuilder<Fields, WC, L, O, Se, So> {
        self.fields.0.push(field.to_owned());
        self
    }

    pub fn select_multiple(mut self, fields: &[String]) -> QueryBuilder<Fields, WC, L, O, Se, So> {
        self.fields.0.extend(fields.to_vec());
        self
    }
}

impl<F, L, O, Se, So> QueryBuilder<F, UnspecifiedOption, L, O, Se, So> {
    pub fn where_(self, clause: &str) -> QueryBuilder<F, WhereClause, L, O, Se, So> {
        QueryBuilder {
            fields: self.fields,
            where_clause: WhereClause::new(clause),
            limit: self.limit,
            offset: self.offset,
            search: self.search,
            sort: self.sort,
        }
    }
}

impl<F, L, O, Se, So> QueryBuilder<F, WhereClause, L, O, Se, So> {
    pub fn and(mut self, clause: &str) -> Self {
        let existing = self.where_clause.0;
        let updated = format!("{} & {}", existing, clause);
        self.where_clause = WhereClause(updated);
        self
    }

    pub fn or(mut self, clause: &str) -> Self {
        let existing = self.where_clause.0;
        let updated = format!("{} | {}", existing, clause);
        self.where_clause = WhereClause(updated);
        self
    }

    pub fn where_(self, clause: &str) -> Self {
        self.and(clause)
    }
}

impl<F, WC, O, Se, So> QueryBuilder<F, WC, UnspecifiedOption, O, Se, So> {
    pub fn limit(self, limit: u32) -> QueryBuilder<F, WC, Limit, O, Se, So> {
        QueryBuilder {
            fields: self.fields,
            where_clause: self.where_clause,
            limit: Limit::new(limit),
            offset: self.offset,
            search: self.search,
            sort: self.sort,
        }
    }
}

impl<F, WC, L, Se, So> QueryBuilder<F, WC, L, UnspecifiedOption, Se, So> {
    pub fn offset(self, offset: u32) -> QueryBuilder<F, WC, L, Offset, Se, So> {
        QueryBuilder {
            fields: self.fields,
            where_clause: self.where_clause,
            limit: self.limit,
            offset: Offset::new(offset),
            search: self.search,
            sort: self.sort,
        }
    }
}

impl<F, WC, L, O, So> QueryBuilder<F, WC, L, O, UnspecifiedOption, So> {
    pub fn search(self, query: &str) -> QueryBuilder<F, WC, L, O, Search, So> {
        QueryBuilder {
            fields: self.fields,
            where_clause: self.where_clause,
            limit: self.limit,
            offset: self.offset,
            search: Search::new(query),
            sort: self.sort,
        }
    }
}

impl<F, WC, L, O, Se> QueryBuilder<F, WC, L, O, Se, UnspecifiedOption> {
    pub fn sort(
        self,
        field: &str,
        direction: SortDirection,
    ) -> QueryBuilder<F, WC, L, O, Se, Sort> {
        QueryBuilder {
            fields: self.fields,
            where_clause: self.where_clause,
            limit: self.limit,
            offset: self.offset,
            search: self.search,
            sort: Sort(field.to_owned(), direction),
        }
    }
}

impl<F, WC, L, O, Se, So> QueryBuilder<F, WC, L, O, Se, So>
where
    F: Display,
    WC: Display,
    L: Display,
    O: Display,
    Se: Display,
    So: Display,
{
    pub fn build(self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.fields, self.where_clause, self.search, self.offset, self.limit, self.sort
        )
        .trim()
        .to_owned()
    }
}

#[cfg(test)]
mod tests {
    use crate::query::SortDirection;

    use super::Query;

    #[test]
    fn select_single_field() {
        let query = Query::select("name").build();
        assert_eq!("fields name;", query)
    }

    #[test]
    fn select_multiple_fields() {
        let fields = ["name", "date"].map(String::from);
        let query = Query::select_multiple(&fields).build();
        assert_eq!("fields name, date;", query)
    }

    #[test]
    fn simple_where_clause() {
        let query = Query::where_("id = 5").build();
        assert_eq!("fields *; where id = 5;", query)
    }

    #[test]
    fn multiple_where_clauses() {
        let query = Query::where_("id = 42").where_("value = 5").build();
        assert_eq!("fields *; where id = 42 & value = 5;", query)
    }

    #[test]
    fn single_and() {
        let query = Query::where_("id = 5").and("value = 42").build();
        assert_eq!("fields *; where id = 5 & value = 42;", query)
    }

    #[test]
    fn multiple_ands() {
        let query = Query::where_("id = 5")
            .and("value = 42")
            .and(r#"name = "Mario""#)
            .build();
        assert_eq!(
            "fields *; where id = 5 & value = 42 & name = \"Mario\";",
            query
        )
    }

    #[test]
    fn single_or() {
        let query = Query::where_("id = 5").or("value = 42").build();
        assert_eq!("fields *; where id = 5 | value = 42;", query)
    }

    #[test]
    fn multiple_ors() {
        let query = Query::where_("id = 5")
            .or("value = 42")
            .or(r#"name = "Mario""#)
            .build();
        assert_eq!(
            "fields *; where id = 5 | value = 42 | name = \"Mario\";",
            query
        )
    }

    #[test]
    fn search() {
        let query = Query::search("Halo").build();
        assert_eq!("fields *; search \"Halo\";", query)
    }

    #[test]
    fn only_limit() {
        let query = Query::limit(10).build();
        assert_eq!("fields *; limit 10;", query);
    }

    #[test]
    fn only_offset() {
        let query = Query::offset(10).build();
        assert_eq!("fields *; offset 10;", query);
    }

    #[test]
    fn sort_ascending() {
        let query = Query::sort("release_dates.date", SortDirection::Ascending).build();
        assert_eq!("fields *; sort release_dates.date asc;", query);
    }

    #[test]
    fn sort_descending() {
        let query = Query::sort("release_dates.date", SortDirection::Descending).build();
        assert_eq!("fields *; sort release_dates.date desc;", query);
    }

    #[test]
    fn complex_query_1() {
        // This is a nonsense query, but meant to test all the paths

        let query = Query::select("name")
            .select("rating")
            .search("God of War")
            .where_("platform = \"PS4\"")
            .and("developer = \"Insomniac\"")
            .or("developer = \"Naughty Dog\"")
            .select_multiple(&["publisher", "release_date.date"].map(String::from))
            .limit(100)
            .offset(5)
            .sort("rating", SortDirection::Ascending)
            .build();
        println!("{}", query)
    }

    #[test]
    fn complex_query_2() {
        // This is a nonsense query, but meant to test all the paths

        let query = Query::search("God of War")
            .select("rating")
            .where_("platform = \"PS4\"")
            .and("developer = \"Insomniac\"")
            .or("developer = \"Naughty Dog\"")
            .limit(100)
            .offset(5)
            .sort("rating", SortDirection::Ascending)
            .build();
        println!("{}", query)
    }

    #[test]
    fn complex_query_3() {
        // This is a nonsense query, but meant to test all the paths

        let query = Query::search("God of War")
            .select_multiple(&["rating", "release_date"].map(String::from))
            .where_("platform = \"PS4\"")
            .and("developer = \"Insomniac\"")
            .or("developer = \"Naughty Dog\"")
            .limit(100)
            .offset(5)
            .sort("rating", SortDirection::Ascending)
            .build();
        println!("{}", query)
    }
}
