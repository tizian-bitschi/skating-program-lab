// The service key envelope is context + whole code + direct-step assumption.
// Compare the opaque whole suffix across categories; never parse its jump grammar.
export const connectionIdentity=(key:string)=>key.split(':').slice(3).join(':');
export const connectionCode=(key:string)=>key.split(':').at(-2)??key;
export const connectionText={
 en:{exclude:'Exclude connection',excluded:'Excluded',scope:'Only this whole connection is excluded. Its solo jumps and longer connections remain available.',sequence:'Direct step into Axel assumed',update:'Update suggestions',outdated:'Outdated',undo:'Undo exclusion',removed:'Removed occurrences',undone:'Exclusion undone; removed occurrences restored.',program:'Program',value:'Credited BV'},
 de:{exclude:'Verbindung ausschließen',excluded:'Ausgeschlossen',scope:'Nur diese ganze Verbindung ist ausgeschlossen. Ihre Einzelsprünge und längere Verbindungen bleiben verfügbar.',sequence:'Direkter Schritt in den Axel angenommen',update:'Vorschläge aktualisieren',outdated:'Veraltet',undo:'Ausschluss rückgängig machen',removed:'Entfernte Vorkommen',undone:'Ausschluss rückgängig gemacht; entfernte Vorkommen wiederhergestellt.',program:'Programm',value:'Angerechneter Basiswert'},
 ru:{exclude:'Исключить соединение',excluded:'Исключено',scope:'Исключается только это соединение целиком. Его одиночные прыжки и более длинные соединения остаются доступны.',sequence:'Предполагается прямой шаг в аксель',update:'Обновить варианты',outdated:'Устарело',undo:'Отменить исключение',removed:'Удалено элементов',undone:'Исключение отменено; удалённые элементы восстановлены.',program:'Программа',value:'Зачтённая базовая стоимость'}
};
