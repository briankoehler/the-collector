use the_collector_ipc::{
    r#pub::IpcPublisher, sub::IpcSubscriber, SummonerMatchQuery, IPC_SUMMONER_MATCH_PATH,
};

#[tokio::test]
async fn test_pub_sub() {
    let original_message = SummonerMatchQuery {
        puuid: "puuid".into(),
        match_id: "match_id".into(),
    };
    let sent_message = original_message.clone();
    let subscriber_task = tokio::task::spawn(async move {
        let subscriber = IpcSubscriber::<SummonerMatchQuery>::new(IPC_SUMMONER_MATCH_PATH).unwrap();
        let message = subscriber.recv().await.unwrap();
        message
    });
    let publisher_task = tokio::task::spawn(async move {
        let publisher = IpcPublisher::<SummonerMatchQuery>::new(IPC_SUMMONER_MATCH_PATH).unwrap();
        publisher.publish(sent_message).await.unwrap();
    });

    let (received_message, _) = tokio::join!(subscriber_task, publisher_task);
    assert_eq!(received_message.unwrap(), original_message);
}
