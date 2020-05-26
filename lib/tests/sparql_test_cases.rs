///! Integration tests based on [SPARQL 1.1 Test Cases](https://www.w3.org/2009/sparql/docs/tests/README.html)
use oxigraph::model::vocab::rdf;
use oxigraph::model::vocab::rdfs;
use oxigraph::model::*;
use oxigraph::sparql::*;
use oxigraph::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::sync::Arc;

#[test]
fn sparql_w3c_syntax_testsuite() -> Result<()> {
    let manifest_10_urls =
        vec!["http://www.w3.org/2001/sw/DataAccess/tests/data-r2/manifest-syntax.ttl"];
    let manifest_11_urls = vec![
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/syntax-query/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/syntax-fed/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/construct/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/grouping/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/aggregates/manifest.ttl",
    ];
    for test_result in manifest_10_urls
        .into_iter()
        .chain(manifest_11_urls.into_iter())
        .flat_map(TestManifest::new)
    {
        let test = test_result.unwrap();
        if test.kind == "PositiveSyntaxTest" || test.kind == "PositiveSyntaxTest11" {
            match Query::parse(&read_file_to_string(&test.query)?, Some(&test.query)) {
                Err(error) => panic!("Failure on {} with error: {}", test, error),
                Ok(query) => {
                    if let Err(error) = Query::parse(&query.to_string(), None) {
                        assert!(
                            false,
                            "Failure to deserialize \"{}\" of {} with error: {}",
                            query.to_string(),
                            test,
                            error
                        )
                    }
                }
            }
        } else if test.kind == "NegativeSyntaxTest" || test.kind == "NegativeSyntaxTest11" {
            //TODO
            if let Ok(result) = Query::parse(&read_file_to_string(&test.query)?, Some(&test.query))
            {
                eprintln!("Failure on {}. The output tree is: {}", test, result);
            }
        } else if test.kind != "QueryEvaluationTest" {
            panic!("Not supported test: {}", test);
        }
    }
    Ok(())
}

#[test]
fn sparql_w3c_query_evaluation_testsuite() -> Result<()> {
    let manifest_10_urls = vec![
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/algebra/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/ask/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/basic/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/bnode-coreference/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/boolean-effective-value/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/bound/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/cast/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/construct/manifest.ttl",
        //TODO FROM and FROM NAMED "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/construct/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/distinct/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-equals/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-ops/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/graph/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/i18n/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/optional/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/optional-filter/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/reduced/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/regex/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/solution-seq/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/sort/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/triple-match/manifest.ttl",
        "http://www.w3.org/2001/sw/DataAccess/tests/data-r2/type-promotion/manifest.ttl",
    ];

    let manifest_11_urls = vec![
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/aggregates/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bind/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/bindings/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/construct/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/exists/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/functions/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/grouping/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/negation/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/project-expression/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/property-path/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/service/manifest.ttl",
        "http://www.w3.org/2009/sparql/docs/tests/data-sparql11/subquery/manifest.ttl",
    ];

    let test_blacklist = vec![
        //Multiple writing of the same xsd:integer. Our system does strong normalization.
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/distinct/manifest#distinct-1").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/distinct/manifest#distinct-9").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#dawg-str-1").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#dawg-str-2").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-equals/manifest#eq-graph-1").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-equals/manifest#eq-graph-2").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-01").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-04").unwrap(),
        //Multiple writing of the same xsd:double. Our system does strong normalization.
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#sameTerm").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#sameTerm-simple").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#sameTerm-eq").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#sameTerm-not-eq").unwrap(),
        //Simple literal vs xsd:string. We apply RDF 1.1
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/distinct/manifest#distinct-2").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-08").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-10").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-11").unwrap(),
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#open-eq-12").unwrap(),
        //DATATYPE("foo"@en) returns rdf:langString in RDF 1.1
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/expr-builtin/manifest#dawg-datatype-2").unwrap(),
        // FROM support
        NamedNodeBuf::parse("http://www.w3.org/2009/sparql/docs/tests/data-sparql11/construct/manifest#constructwhere04").unwrap(),
        //BNODE() scope is currently wrong
        NamedNodeBuf::parse("http://www.w3.org/2009/sparql/docs/tests/data-sparql11/functions/manifest#bnode01").unwrap(),
        //Property path with unbound graph name are not supported yet
        NamedNodeBuf::parse("http://www.w3.org/2009/sparql/docs/tests/data-sparql11/property-path/manifest#pp35").unwrap(),
        //SERVICE name from a BGP
        NamedNodeBuf::parse("http://www.w3.org/2009/sparql/docs/tests/data-sparql11/service/manifest#service5").unwrap(),
        // We use XSD 1.1 equality on dates
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/open-world/manifest#date-2").unwrap(),
        // We choose to simplify first the nested group patterns in OPTIONAL
        NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/optional-filter/manifest#dawg-optional-filter-005-not-simplified").unwrap(),
    ];

    let tests: Result<Vec<_>> = manifest_10_urls
        .into_iter()
        .chain(manifest_11_urls.into_iter())
        .flat_map(TestManifest::new)
        .collect();
    let failed: Vec<_> = tests?.into_par_iter().map(|test| {
        if test_blacklist.contains(&test.id) {
            Ok(())
        } else if test.kind == "QueryEvaluationTest" {
            let store = MemoryStore::default();
            if let Some(data) = &test.data {
                load_graph_to_store(&data, &store, None)?;
            }
            for graph_data in &test.graph_data {
                load_graph_to_store(
                    &graph_data,
                    &store,
                    Some(&NamedNodeBuf::parse(graph_data)?.into()),
                )?;
            }
            match store.prepare_query(&read_file_to_string(&test.query)?, QueryOptions::default().with_base_iri(&test.query).with_service_handler(StaticServiceHandler::new(&test.service_data)?))
                {
                    Err(error) => Err(Error::msg(format!(
                    "Failure to parse query of {} with error: {}",
                    test, error
                ))),
                    Ok(query) => match query.exec() {
                        Err(error) => Err(Error::msg(format!(
                        "Failure to execute query of {} with error: {}",
                        test, error
                    ))),
                        Ok(result) => {
                            let expected_graph =
                                load_sparql_query_result_graph(test.result.as_ref().unwrap()).map_err(|e| Error::msg(format!("Error constructing expected graph for {}: {}", test, e)))?;
                            let with_order = expected_graph
                                .triples_for_predicate(&rs::INDEX)
                                .next()
                                .is_some();
                            let actual_graph = to_graph(result, with_order).map_err(|e| Error::msg(format!("Error constructing result graph for {}: {}", test, e)))?;
                            if actual_graph.is_isomorphic(&expected_graph) {
                                Ok(())
                            } else {
                                Err(Error::msg(format!("Failure on {}.\nExpected file:\n{}\nOutput file:\n{}\nParsed query:\n{}\nData:\n{}\n",
                                test,
                                expected_graph,
                                actual_graph,
                                Query::parse(&read_file_to_string(&test.query)?, Some(&test.query)).unwrap(),
                                store_to_string(&store)
                            )))
                            }
                        }
                    },
                }
        } else if test.kind != "NegativeSyntaxTest11" {
            panic!("Not supported test: {}", test)
        } else {
            Ok(())
        }
    }).filter_map(|v| v.err()).map(|e| e.to_string()).collect();
    assert!(
        failed.is_empty(),
        "{} tests failed:\n{}",
        failed.len(),
        failed.join("\n")
    );
    Ok(())
}

fn store_to_string(store: &MemoryStore) -> String {
    store
        .quads_for_pattern(None, None, None, None)
        .map(|q| q.unwrap().to_string() + "\n")
        .collect()
}

fn load_graph(url: &str) -> Result<SimpleGraph> {
    let store = MemoryStore::default();
    load_graph_to_store(url, &store, None)?;
    Ok(store
        .quads_for_pattern(None, None, None, Some(None))
        .map(|q| q.unwrap().into_triple())
        .collect())
}

fn load_graph_to_store(
    url: &str,
    store: &MemoryStore,
    to_graph_name: Option<&NamedOrBlankNode>,
) -> Result<()> {
    let syntax = if url.ends_with(".nt") {
        GraphSyntax::NTriples
    } else if url.ends_with(".ttl") {
        GraphSyntax::Turtle
    } else if url.ends_with(".rdf") {
        GraphSyntax::RdfXml
    } else {
        return Err(Error::msg(format!(
            "Serialization type not found for {}",
            url
        )));
    };
    store.load_graph(read_file(url)?, syntax, to_graph_name, Some(url))
}

fn load_sparql_query_result_graph(url: &str) -> Result<SimpleGraph> {
    let store = MemoryStore::default();
    if url.ends_with(".srx") {
        for t in to_graph(
            QueryResult::read(read_file(url)?, QueryResultSyntax::Xml)?,
            false,
        )? {
            store.insert(&t.in_graph(None))?;
        }
    } else {
        load_graph_to_store(url, &store, None)?;
    }
    Ok(store
        .quads_for_pattern(None, None, None, Some(None))
        .map(|q| q.unwrap().into_triple())
        .collect())
}

fn to_relative_path(url: &str) -> Result<String> {
    if url.starts_with("http://www.w3.org/2001/sw/DataAccess/tests/data-r2/") {
        Ok(url.replace(
            "http://www.w3.org/2001/sw/DataAccess/tests/",
            "rdf-tests/sparql11/",
        ))
    } else if url.starts_with("http://www.w3.org/2009/sparql/docs/tests/data-sparql11/") {
        Ok(url.replace(
            "http://www.w3.org/2009/sparql/docs/tests/",
            "rdf-tests/sparql11/",
        ))
    } else {
        Err(Error::msg(format!("Not supported url for file: {}", url)))
    }
}

fn read_file(url: &str) -> Result<impl BufRead> {
    let mut base_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    base_path.push("tests");
    base_path.push(to_relative_path(url)?);

    Ok(BufReader::new(File::open(&base_path).map_err(|e| {
        Error::msg(format!(
            "Opening file {} failed with {}",
            base_path.display(),
            e,
        ))
    })?))
}

fn read_file_to_string(url: &str) -> Result<String> {
    let mut string = String::default();
    read_file(url)?.read_to_string(&mut string)?;
    Ok(string)
}

mod rs {
    use lazy_static::lazy_static;
    use oxigraph::model::NamedNodeBuf;

    lazy_static! {
        pub static ref RESULT_SET: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/result-set#ResultSet")
                .unwrap();
        pub static ref RESULT_VARIABLE: NamedNodeBuf = NamedNodeBuf::parse(
            "http://www.w3.org/2001/sw/DataAccess/tests/result-set#resultVariable"
        )
        .unwrap();
        pub static ref SOLUTION: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/result-set#solution")
                .unwrap();
        pub static ref BINDING: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/result-set#binding")
                .unwrap();
        pub static ref VALUE: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/result-set#value")
                .unwrap();
        pub static ref VARIABLE: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/result-set#variable")
                .unwrap();
        pub static ref INDEX: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/result-set#index")
                .unwrap();
        pub static ref BOOLEAN: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/result-set#boolean")
                .unwrap();
    }
}

fn to_graph(result: QueryResult<'_>, with_order: bool) -> Result<SimpleGraph> {
    match result {
        QueryResult::Graph(graph) => graph.collect(),
        QueryResult::Boolean(value) => {
            let mut graph = SimpleGraph::default();
            let result_set = BlankNode::default();
            graph.insert(Triple::new(
                result_set,
                rdf::TYPE.clone(),
                rs::RESULT_SET.clone(),
            ));
            graph.insert(Triple::new(
                result_set,
                rs::BOOLEAN.clone(),
                Literal::from(value),
            ));
            Ok(graph)
        }
        QueryResult::Bindings(bindings) => {
            let mut graph = SimpleGraph::default();
            let result_set = BlankNode::default();
            graph.insert(Triple::new(
                result_set,
                rdf::TYPE.clone(),
                rs::RESULT_SET.clone(),
            ));
            let (variables, iter) = bindings.destruct();
            for variable in &variables {
                graph.insert(Triple::new(
                    result_set,
                    rs::RESULT_VARIABLE.clone(),
                    Literal::new_simple_literal(variable.name()?),
                ));
            }
            for (i, binding_values) in iter.enumerate() {
                let binding_values = binding_values?;
                let solution = BlankNode::default();
                graph.insert(Triple::new(result_set, rs::SOLUTION.clone(), solution));
                for i in 0..variables.len() {
                    if let Some(ref value) = binding_values[i] {
                        let binding = BlankNode::default();
                        graph.insert(Triple::new(solution, rs::BINDING.clone(), binding));
                        graph.insert(Triple::new(binding, rs::VALUE.clone(), value.clone()));
                        graph.insert(Triple::new(
                            binding,
                            rs::VARIABLE.clone(),
                            Literal::new_simple_literal(variables[i].name()?),
                        ));
                    }
                }
                if with_order {
                    graph.insert(Triple::new(
                        solution,
                        rs::INDEX.clone(),
                        Literal::from((i + 1) as i128),
                    ));
                }
            }
            Ok(graph)
        }
    }
}

pub struct Test {
    pub id: NamedNodeBuf,
    pub kind: String,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub query: String,
    pub data: Option<String>,
    pub graph_data: Vec<String>,
    pub service_data: Vec<(String, String)>,
    pub result: Option<String>,
}

impl fmt::Display for Test {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)?;
        for name in &self.name {
            write!(f, " named \"{}\"", name)?;
        }
        for comment in &self.comment {
            write!(f, " with comment \"{}\"", comment)?;
        }
        write!(f, " on query {}", self.query)?;
        for data in &self.data {
            write!(f, " with data {}", data)?;
        }
        for data in &self.graph_data {
            write!(f, " and graph data {}", data)?;
        }
        for result in &self.result {
            write!(f, " and expected result {}", result)?;
        }
        Ok(())
    }
}

pub struct TestManifest {
    graph: SimpleGraph,
    tests_to_do: Vec<Term>,
    manifests_to_do: Vec<String>,
}

impl TestManifest {
    pub fn new(url: impl Into<String>) -> TestManifest {
        Self {
            graph: SimpleGraph::default(),
            tests_to_do: Vec::default(),
            manifests_to_do: vec![url.into()],
        }
    }
}

pub mod mf {
    use lazy_static::lazy_static;
    use oxigraph::model::NamedNodeBuf;

    lazy_static! {
        pub static ref INCLUDE: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#include")
                .unwrap();
        pub static ref ENTRIES: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#entries")
                .unwrap();
        pub static ref NAME: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#name")
                .unwrap();
        pub static ref ACTION: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#action")
                .unwrap();
        pub static ref RESULT: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-manifest#result")
                .unwrap();
    }
}

pub mod qt {
    use lazy_static::lazy_static;
    use oxigraph::model::NamedNodeBuf;

    lazy_static! {
        pub static ref QUERY: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-query#query")
                .unwrap();
        pub static ref DATA: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-query#data")
                .unwrap();
        pub static ref GRAPH_DATA: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-query#graphData")
                .unwrap();
        pub static ref SERVICE_DATA: NamedNodeBuf = NamedNodeBuf::parse(
            "http://www.w3.org/2001/sw/DataAccess/tests/test-query#serviceData"
        )
        .unwrap();
        pub static ref ENDPOINT: NamedNodeBuf =
            NamedNodeBuf::parse("http://www.w3.org/2001/sw/DataAccess/tests/test-query#endpoint")
                .unwrap();
    }
}

impl Iterator for TestManifest {
    type Item = Result<Test>;

    fn next(&mut self) -> Option<Result<Test>> {
        match self.tests_to_do.pop() {
            Some(Term::NamedNode(test_node)) => {
                let test_subject = NamedOrBlankNode::from(test_node.clone());
                let kind = match self
                    .graph
                    .object_for_subject_predicate(&test_subject, &rdf::TYPE)
                {
                    Some(Term::NamedNode(c)) => match c.as_str().split('#').last() {
                        Some(k) => k.to_string(),
                        None => return self.next(), //We ignore the test
                    },
                    _ => return self.next(), //We ignore the test
                };
                let name = match self
                    .graph
                    .object_for_subject_predicate(&test_subject, &mf::NAME)
                {
                    Some(Term::Literal(c)) => Some(c.value().to_string()),
                    _ => None,
                };
                let comment = match self
                    .graph
                    .object_for_subject_predicate(&test_subject, &rdfs::COMMENT)
                {
                    Some(Term::Literal(c)) => Some(c.value().to_string()),
                    _ => None,
                };
                let (query, data, graph_data, service_data) = match self
                    .graph
                    .object_for_subject_predicate(&test_subject, &*mf::ACTION)
                {
                    Some(Term::NamedNode(n)) => (n.as_str().to_owned(), None, vec![], vec![]),
                    Some(Term::BlankNode(n)) => {
                        let n = n.clone().into();
                        let query = match self.graph.object_for_subject_predicate(&n, &qt::QUERY) {
                            Some(Term::NamedNode(q)) => q.as_str().to_owned(),
                            Some(_) => return Some(Err(Error::msg("invalid query"))),
                            None => return Some(Err(Error::msg("query not found"))),
                        };
                        let data = match self.graph.object_for_subject_predicate(&n, &qt::DATA) {
                            Some(Term::NamedNode(q)) => Some(q.as_str().to_owned()),
                            _ => None,
                        };
                        let graph_data = self
                            .graph
                            .objects_for_subject_predicate(&n, &qt::GRAPH_DATA)
                            .filter_map(|g| match g {
                                Term::NamedNode(q) => Some(q.as_str().to_owned()),
                                _ => None,
                            })
                            .collect();
                        let service_data = self
                            .graph
                            .objects_for_subject_predicate(&n, &qt::SERVICE_DATA)
                            .filter_map(|g| match g {
                                Term::NamedNode(g) => Some(g.clone().into()),
                                Term::BlankNode(g) => Some(g.clone().into()),
                                _ => None,
                            })
                            .filter_map(|g| {
                                if let (
                                    Some(Term::NamedNode(endpoint)),
                                    Some(Term::NamedNode(data)),
                                ) = (
                                    self.graph.object_for_subject_predicate(&g, &qt::ENDPOINT),
                                    self.graph.object_for_subject_predicate(&g, &qt::DATA),
                                ) {
                                    Some((endpoint.as_str().to_owned(), data.as_str().to_owned()))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        (query, data, graph_data, service_data)
                    }
                    Some(_) => return Some(Err(Error::msg("invalid action"))),
                    None => {
                        return Some(Err(Error::msg(format!(
                            "action not found for test {}",
                            test_subject
                        ))));
                    }
                };
                let result = match self
                    .graph
                    .object_for_subject_predicate(&test_subject, &*mf::RESULT)
                {
                    Some(Term::NamedNode(n)) => Some(n.as_str().to_owned()),
                    Some(_) => return Some(Err(Error::msg("invalid result"))),
                    None => None,
                };
                Some(Ok(Test {
                    id: test_node,
                    kind,
                    name,
                    comment,
                    query,
                    data,
                    graph_data,
                    service_data,
                    result,
                }))
            }
            Some(_) => Some(Err(Error::msg("invalid test list"))),
            None => {
                match self.manifests_to_do.pop() {
                    Some(url) => {
                        let manifest =
                            NamedOrBlankNode::from(NamedNodeBuf::parse(url.clone()).unwrap());
                        match load_graph(&url) {
                            Ok(g) => self.graph.extend(g.into_iter()),
                            Err(e) => return Some(Err(e)),
                        }

                        // New manifests
                        match self
                            .graph
                            .object_for_subject_predicate(&manifest, &*mf::INCLUDE)
                        {
                            Some(Term::BlankNode(list)) => {
                                self.manifests_to_do.extend(
                                    RdfListIterator::iter(&self.graph, list.clone().into())
                                        .filter_map(|m| match m {
                                            Term::NamedNode(nm) => Some(nm.into_string()),
                                            _ => None,
                                        }),
                                );
                            }
                            Some(_) => return Some(Err(Error::msg("invalid tests list"))),
                            None => (),
                        }

                        // New tests
                        match self
                            .graph
                            .object_for_subject_predicate(&manifest, &*mf::ENTRIES)
                        {
                            Some(Term::BlankNode(list)) => {
                                self.tests_to_do.extend(RdfListIterator::iter(
                                    &self.graph,
                                    list.clone().into(),
                                ));
                            }
                            Some(term) => {
                                return Some(Err(Error::msg(format!(
                                    "Invalid tests list. Got term {}",
                                    term
                                ))));
                            }
                            None => (),
                        }
                    }
                    None => return None,
                }
                self.next()
            }
        }
    }
}

pub struct RdfListIterator<'a> {
    graph: &'a SimpleGraph,
    current_node: Option<NamedOrBlankNode>,
}

impl<'a> RdfListIterator<'a> {
    fn iter(graph: &'a SimpleGraph, root: NamedOrBlankNode) -> RdfListIterator<'a> {
        RdfListIterator {
            graph,
            current_node: Some(root),
        }
    }
}

impl<'a> Iterator for RdfListIterator<'a> {
    type Item = Term;

    fn next(&mut self) -> Option<Term> {
        match self.current_node.clone() {
            Some(current) => {
                let result = self
                    .graph
                    .object_for_subject_predicate(&current, &rdf::FIRST);
                self.current_node = match self
                    .graph
                    .object_for_subject_predicate(&current, &rdf::REST)
                {
                    Some(Term::NamedNode(ref n)) if *n == *rdf::NIL => None,
                    Some(Term::NamedNode(n)) => Some(n.clone().into()),
                    Some(Term::BlankNode(n)) => Some(n.clone().into()),
                    _ => None,
                };
                result.cloned()
            }
            None => None,
        }
    }
}

#[derive(Clone)]
struct StaticServiceHandler {
    services: Arc<HashMap<NamedNodeBuf, MemoryStore>>,
}

impl StaticServiceHandler {
    fn new(services: &[(String, String)]) -> Result<Self> {
        Ok(Self {
            services: Arc::new(
                services
                    .iter()
                    .map(|(name, data)| {
                        let name = NamedNodeBuf::parse(name)?;
                        let store = MemoryStore::default();
                        load_graph_to_store(&data, &store, None)?;
                        Ok((name, store))
                    })
                    .collect::<Result<_>>()?,
            ),
        })
    }
}

impl ServiceHandler for StaticServiceHandler {
    fn handle<'a>(
        &'a self,
        service_name: &NamedNode,
        graph_pattern: &'a GraphPattern,
    ) -> Result<BindingsIterator<'a>> {
        if let QueryResult::Bindings(iterator) = self
            .services
            .get(service_name)
            .ok_or_else(|| Error::msg(format!("Service {} not found", service_name)))?
            .prepare_query_from_pattern(
                &graph_pattern,
                QueryOptions::default().with_service_handler(self.clone()),
            )?
            .exec()?
        {
            //TODO: very hugly
            let (variables, iter) = iterator.destruct();
            let collected = iter.collect::<Vec<_>>();
            Ok(BindingsIterator::new(
                variables,
                Box::new(collected.into_iter()),
            ))
        } else {
            Err(Error::msg("Expected bindings but got another QueryResult"))
        }
    }
}
