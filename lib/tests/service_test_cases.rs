use oxigraph::model::*;
use oxigraph::sparql::*;
use oxigraph::*;
use std::io::BufRead;

#[test]
fn simple_service_test() {
    struct TestServiceHandler;
    impl ServiceHandler for TestServiceHandler {
        fn handle<'a>(
            &'a self,
            _: &NamedNode,
            graph_pattern: &'a GraphPattern,
        ) -> Result<BindingsIterator<'a>> {
            let triples =
                b"<http://example.com/s> <http://example.com/p> <http://example.com/o> .".as_ref();
            do_pattern(triples, graph_pattern, QueryOptions::default())
        }
    }

    let query = r#"
  SELECT ?s ?p ?o
  WHERE
    { 
      SERVICE <http://service1.org>
      { ?s ?p ?o
      }
   }
  "#
    .to_string();

    let options = QueryOptions::default().with_service_handler(TestServiceHandler);
    let results = do_query(b"".as_ref(), query, options).unwrap();
    let collected = results
        .into_values_iter()
        .map(move |b| b.unwrap())
        .collect::<Vec<_>>();
    let solution = vec![vec![
        Some(ex(String::from("s"))),
        Some(ex(String::from("p"))),
        Some(ex(String::from("o"))),
    ]];
    assert_eq!(collected, solution);
}

#[test]
fn two_service_test() {
    #[derive(Clone, Copy)]
    struct TwoServiceTest;
    impl ServiceHandler for TwoServiceTest {
        fn handle<'a>(
            &'a self,
            named_node: &NamedNode,
            graph_pattern: &'a GraphPattern,
        ) -> Result<BindingsIterator<'a>> {
            let service1 = NamedNode::parse("http://service1.org").unwrap();
            let service2 = NamedNode::parse("http://service2.org").unwrap();
            if named_node == service1 {
                let triples = br#"
        <http://example.com/bob> <http://xmlns.com/foaf/0.1/name> "Bob" .
        <http://example.com/alice> <http://xmlns.com/foaf/0.1/name> "Alice" .
        "#
                .as_ref();
                do_pattern(triples, graph_pattern, QueryOptions::default())
            } else if named_node == service2 {
                let triples = br#"
        <http://example.com/bob> <http://xmlns.com/foaf/0.1/mbox> <mailto:bob@example.com> .
        <http://example.com/alice> <http://xmlns.com/foaf/0.1/mbox> <mailto:alice@example.com> .
        "#
                .as_ref();
                do_pattern(triples, graph_pattern, QueryOptions::default())
            } else {
                Err(Error::msg("not found"))
            }
        }
    }

    let query = r#"
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name ?mbox 
  WHERE
    { 
      SERVICE <http://service1.org>
      { ?s foaf:name ?name
      }

      SERVICE <http://service2.org>
      { ?s foaf:mbox ?mbox
      }
    }
  ORDER BY ?name
  "#
    .to_string();

    let options = QueryOptions::default().with_service_handler(TwoServiceTest);
    let results = do_query(b"".as_ref(), query, options).unwrap();
    let collected = results
        .into_values_iter()
        .map(move |b| b.unwrap())
        .collect::<Vec<_>>();
    let solution = vec![
        vec![
            Some(literal("Alice".to_string())),
            Some(mailto("alice@example.com".to_string())),
        ],
        vec![
            Some(literal("Bob".to_string())),
            Some(mailto("bob@example.com".to_string())),
        ],
    ];
    assert_eq!(collected, solution);
}

#[test]
fn silent_service_empty_set_test() {
    #[derive(Clone, Copy)]
    struct ServiceTest;
    impl ServiceHandler for ServiceTest {
        fn handle<'a>(
            &'a self,
            _: &NamedNode,
            _: &'a GraphPattern,
        ) -> Result<BindingsIterator<'a>> {
            Err(Error::msg("This is supposed to fail"))
        }
    }

    let query = r#"
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name ?mbox 
  WHERE
    { 
      SERVICE SILENT <http://service1.org>
      { ?s foaf:name ?name
      }
      
    }
  ORDER BY ?name
  "#
    .to_string();

    let triples = b"".as_ref();
    let options = QueryOptions::default().with_service_handler(ServiceTest);
    let results = do_query(triples, query, options).unwrap();
    let collected = results
        .into_values_iter()
        .map(move |b| b.unwrap())
        .collect::<Vec<_>>();
    assert_eq!(collected.len(), 1);
}

#[test]
fn non_silent_service_test() {
    #[derive(Clone, Copy)]
    struct ServiceTest;
    impl ServiceHandler for ServiceTest {
        fn handle<'a>(
            &'a self,
            _: &NamedNode,
            _: &'a GraphPattern,
        ) -> Result<BindingsIterator<'a>> {
            Err(Error::msg("This is supposed to fail"))
        }
    }

    let query = r#"
  PREFIX foaf: <http://xmlns.com/foaf/0.1/>
  SELECT ?name
  WHERE
    { 
      SERVICE <http://service1.org>
      { ?s foaf:name ?name
      }
      
    }
  ORDER BY ?name
  "#
    .to_string();

    let triples = b"".as_ref();
    let options = QueryOptions::default().with_service_handler(ServiceTest);
    let results = do_query(triples, query, options).unwrap();
    let result = results.into_values_iter().next();
    match result {
        Some(Err(_)) => assert_eq!(true, true),
        _ => assert_eq!(
            true, false,
            "This should have been an error since the service fails"
        ),
    }
}

fn ex(id: String) -> Term {
    Term::NamedNode(NamedNodeBuf::parse(format!("http://example.com/{}", &id)).unwrap())
}

fn mailto(id: String) -> Term {
    Term::NamedNode(NamedNodeBuf::parse(format!("mailto:{}", &id)).unwrap())
}

fn literal(str: String) -> Term {
    Term::Literal(Literal::new_simple_literal(str))
}

fn make_store(reader: impl BufRead) -> Result<MemoryStore> {
    let store = MemoryStore::default();
    store
        .load_graph(reader, GraphSyntax::NTriples, None, None)
        .unwrap();
    Ok(store)
}

fn query_store<'a>(
    store: MemoryStore,
    query: String,
    options: QueryOptions<'a>,
) -> Result<BindingsIterator<'a>> {
    match store.prepare_query(&query, options)?.exec()? {
        QueryResult::Bindings(iterator) => {
            let (varaibles, iter) = iterator.destruct();
            let collected = iter.collect::<Vec<_>>();
            Ok(BindingsIterator::new(
                varaibles,
                Box::new(collected.into_iter()),
            ))
        }
        _ => Err(Error::msg("Excpected bindings but got another QueryResult")),
    }
}

fn pattern_store<'a>(
    store: MemoryStore,
    pattern: &'a GraphPattern,
    options: QueryOptions<'a>,
) -> Result<BindingsIterator<'a>> {
    match store
        .prepare_query_from_pattern(&pattern, options)?
        .exec()?
    {
        QueryResult::Bindings(iterator) => {
            let (varaibles, iter) = iterator.destruct();
            let collected = iter.collect::<Vec<_>>();
            Ok(BindingsIterator::new(
                varaibles,
                Box::new(collected.into_iter()),
            ))
        }
        _ => Err(Error::msg("Expected bindings but got another QueryResult")),
    }
}

fn do_query<'a>(
    reader: impl BufRead,
    query: String,
    options: QueryOptions<'a>,
) -> Result<BindingsIterator<'a>> {
    let store = make_store(reader)?;
    query_store(store, query, options)
}

fn do_pattern<'a>(
    reader: impl BufRead,
    pattern: &'a GraphPattern,
    options: QueryOptions<'a>,
) -> Result<BindingsIterator<'a>> {
    let store = make_store(reader)?;
    pattern_store(store, pattern, options)
}
