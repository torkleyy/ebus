var searchIndex = JSON.parse('{\
"energy_bus":{"doc":"eBUS","t":"DDDQNNNDNNENNNDNNDNEDNILLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLMMLLMLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLMMMKLLLLLLLLLLLLLLLLLLLLLLLLLLLLMMM","n":["Buffer","Crc","EbusDriver","Error","ExpectReply","MasterAckErr","MasterAckOk","MasterTelegram","NeedsDataCrc","None","ProcessResult","Reply","ReplyCrcError","Request","RequestToken","SlaveAckErr","SlaveAckOk","Telegram","TelegramCrcError","TelegramFlag","TelegramFlags","Timeout","Transmit","add","add_decoded","add_multiple","as_bytes","as_bytes_mut","as_reply","as_request","bitand","bitor","bitor","bitor","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","calc_crc","clone","clone","clone","clone","clone","data","dest","eq","eq","flags","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from","from_parts","from_slice","into","into","into","into","into","into","into","into","into","is_none","new","new","none","process","reply_as_slave","reset_syn","reset_wait_syn","service","src","telegram","transmit_raw","transmit_syn","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","data","telegram","token"],"q":[[0,"energy_bus"],[130,"energy_bus::ProcessResult"]],"d":["","","","","Whether or not to wait for the recipient to reply","We sent master-slave, slave did not acknowledge","We sent master-slave, slave acknowledged","Telegram to be sent","Whether the data is expected to have an additional CRC …","","","Slave sent reply","CRC check of reply failed (sent by another slave)","Master-slave request","","We replied as slave, master did not acknowledge","We replied as slave, master acknowledged","","CRC check of telegram failed (sent by another master)","","","Expected recipient to send, but AUTO-SYN occurred","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","Up to 16 data bytes","ZZ - destination eBUS address","","","Options for the handling of this telegram","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","Create <code>Buffer</code> from byte slice with at most 16 elements.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","","","","","","Reply to a received master-slave telegram","this should be called if we receive SYN","","Service command, encoded LSB first","QQ - source eBUS address","Core telegram data","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","","",""],"i":[0,0,0,18,9,5,5,0,9,5,0,5,5,5,0,5,5,0,5,0,0,5,0,1,1,1,4,4,5,5,8,9,9,8,1,17,10,7,4,9,8,5,14,1,17,10,7,4,9,8,5,14,1,10,7,4,9,8,7,7,9,8,10,10,7,4,9,8,5,14,1,17,10,7,4,9,8,5,14,4,4,1,17,10,7,4,9,8,5,14,5,1,17,8,17,17,17,17,7,7,10,18,18,1,17,10,7,4,9,8,5,14,1,17,10,7,4,9,8,5,14,1,17,10,7,4,9,8,5,14,22,23,23],"f":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[[1,2],1],[[1,[3,[2]]],1],[[1,[3,[2]]],1],[4,[[3,[2]]]],[4,[[3,[2]]]],[5,[[6,[[3,[2]]]]]],[5,[[6,[7]]]],[[8,9]],[[9,9]],[[9,8]],[[8,9]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[1,2],[10,10],[7,7],[4,4],[9,9],[8,8],0,0,[[9,9],11],[[8,8],11],0,[[10,12],13],[[7,12],13],[[4,12],13],[[9,12],13],[[8,12],13],[[5,12],13],[[14,12],13],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[[15,[2]],2],4],[[[3,[2]]],4],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[5,11],[2,1],[[16,2,2],17],[[],8],[[17,2,18,19,[6,[10]]],[[20,[5]]]],[[17,[3,[2]],18,19,14],20],[17],[17],0,0,0,[[[3,[2]]],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],20],[[],21],[[],21],[[],21],[[],21],[[],21],[[],21],[[],21],[[],21],[[],21],0,0,0],"c":[],"p":[[3,"Crc"],[15,"u8"],[15,"slice"],[3,"Buffer"],[4,"ProcessResult"],[4,"Option"],[3,"Telegram"],[3,"TelegramFlags"],[4,"TelegramFlag"],[3,"MasterTelegram"],[15,"bool"],[3,"Formatter"],[6,"Result"],[3,"RequestToken"],[15,"array"],[3,"Duration"],[3,"EbusDriver"],[8,"Transmit"],[8,"Fn"],[4,"Result"],[3,"TypeId"],[13,"Reply"],[13,"Request"]]},\
"log":{"doc":"A lightweight logging facade.","t":"NNNNNNEEIDDNDDDRDNNNNLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLLOLLKLLLLLLLOLLLLKLLLLLLLLLLLLLLLLLLLLLLLLLLOLLLLLLLLLLLLLLLLKOOFLLFLLLLLLLLLLLLLLFFFFLLLLLLOLLLLLLLLLLLLLLLLLLLLLLLLO","n":["Debug","Debug","Error","Error","Info","Info","Level","LevelFilter","Log","Metadata","MetadataBuilder","Off","ParseLevelError","Record","RecordBuilder","STATIC_MAX_LEVEL","SetLoggerError","Trace","Trace","Warn","Warn","args","args","as_str","as_str","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","borrow_mut","build","build","builder","builder","clone","clone","clone","clone","cmp","cmp","cmp","cmp","debug","default","default","enabled","eq","eq","eq","eq","eq","eq","eq","error","file","file","file_static","file_static","flush","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","fmt","from","from","from","from","from","from","from","from","from_str","from_str","hash","hash","hash","hash","info","into","into","into","into","into","into","into","into","iter","iter","level","level","level","level","line","line","log","log","log_enabled","logger","max","max","max_level","metadata","metadata","module_path","module_path","module_path_static","module_path_static","new","new","partial_cmp","partial_cmp","partial_cmp","partial_cmp","partial_cmp","partial_cmp","set_logger","set_logger_racy","set_max_level","set_max_level_racy","target","target","target","target","to_level","to_level_filter","trace","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_from","try_into","try_into","try_into","try_into","try_into","try_into","try_into","try_into","type_id","type_id","type_id","type_id","type_id","type_id","type_id","type_id","warn"],"q":[[0,"log"]],"d":["The “debug” level.","Corresponds to the <code>Debug</code> log level.","The “error” level.","Corresponds to the <code>Error</code> log level.","The “info” level.","Corresponds to the <code>Info</code> log level.","An enum representing the available verbosity levels of the …","An enum representing the available verbosity level filters …","A trait encapsulating the operations required of a logger.","Metadata about a log message.","Builder for <code>Metadata</code>.","A level lower than all log levels.","The type returned by <code>from_str</code> when the string doesn’t …","The “payload” of a log message.","Builder for <code>Record</code>.","The statically resolved maximum log level.","The type returned by <code>set_logger</code> if <code>set_logger</code> has already …","The “trace” level.","Corresponds to the <code>Trace</code> log level.","The “warn” level.","Corresponds to the <code>Warn</code> log level.","The message body.","Set <code>args</code>.","Returns the string representation of the <code>Level</code>.","Returns the string representation of the <code>LevelFilter</code>.","","","","","","","","","","","","","","","","","Invoke the builder and return a <code>Record</code>","Returns a <code>Metadata</code> object.","Returns a new builder.","Returns a new builder.","","","","","","","","","Logs a message at the debug level.","","","Determines if a log message with the specified metadata …","","","","","","","","Logs a message at the error level.","The source file containing the message.","Set <code>file</code>","The module path of the message, if it is a <code>&#39;static</code> string.","Set <code>file</code> to a <code>&#39;static</code> string.","Flushes any buffered records.","","","","","","","","","","","","","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","Returns the argument unchanged.","","","","","","","Logs a message at the info level.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Calls <code>U::from(self)</code>.","Iterate through all supported logging levels.","Iterate through all supported filtering levels.","The verbosity level of the message.","Set <code>Metadata::level</code>.","The verbosity level of the message.","Setter for <code>level</code>.","The line containing the message.","Set <code>line</code>","Logs the <code>Record</code>.","The standard logging macro.","Determines if a message logged at the specified level in …","Returns a reference to the logger.","Returns the most verbose logging level.","Returns the most verbose logging level filter.","Returns the current maximum log level.","Metadata about the log directive.","Set <code>metadata</code>. Construct a <code>Metadata</code> object with …","The module path of the message.","Set <code>module_path</code>","The module path of the message, if it is a <code>&#39;static</code> string.","Set <code>module_path</code> to a <code>&#39;static</code> string","Construct new <code>RecordBuilder</code>.","Construct a new <code>MetadataBuilder</code>.","","","","","","","Sets the global logger to a <code>&amp;&#39;static Log</code>.","A thread-unsafe version of <code>set_logger</code>.","Sets the global maximum log level.","A thread-unsafe version of <code>set_max_level</code>.","The name of the target of the directive.","Set <code>Metadata::target</code>","The name of the target of the directive.","Setter for <code>target</code>.","Converts <code>self</code> to the equivalent <code>Level</code>.","Converts the <code>Level</code> to the equivalent <code>LevelFilter</code>.","Logs a message at the trace level.","","","","","","","","","","","","","","","","","","","","","","","","","Logs a message at the warn level."],"i":[4,6,4,6,4,6,0,0,0,0,0,6,0,0,0,0,0,4,6,4,6,1,3,4,6,4,6,1,3,8,7,15,11,4,6,1,3,8,7,15,11,3,7,1,8,4,6,1,8,4,6,8,7,0,3,7,20,4,4,6,6,8,7,11,0,1,3,1,3,20,4,4,6,6,1,3,8,7,15,15,11,11,4,6,1,3,8,7,15,11,4,6,4,6,8,7,0,4,6,1,3,8,7,15,11,4,6,1,3,8,7,1,3,20,0,0,0,4,6,0,1,3,1,3,1,3,3,7,4,4,6,6,8,7,0,0,0,0,1,3,8,7,6,4,0,4,6,1,3,8,7,15,11,4,6,1,3,8,7,15,11,4,6,1,3,8,7,15,11,0],"f":[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,[1,2],[[3,2],3],[4,5],[6,5],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[3,1],[7,8],[[],3],[[],7],[4,4],[6,6],[1,1],[8,8],[[4,4],9],[[6,6],9],[[8,8],9],[[7,7],9],0,[[],3],[[],7],[8,10],[[4,6],10],[[4,4],10],[[6,4],10],[[6,6],10],[[8,8],10],[[7,7],10],[[11,11],10],0,[1,[[12,[5]]]],[[3,[12,[5]]],3],[1,[[12,[5]]]],[[3,[12,[5]]],3],[[]],[[4,13],14],[[4,13],14],[[6,13],14],[[6,13],14],[[1,13],14],[[3,13],14],[[8,13],14],[[7,13],14],[[15,13],14],[[15,13],14],[[11,13],14],[[11,13],14],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[5,[[16,[4]]]],[5,[[16,[6]]]],[[4,17]],[[6,17]],[[8,17]],[[7,17]],0,[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[]],[[],18],[[],18],[1,4],[[3,4],3],[8,4],[[7,4],7],[1,[[12,[19]]]],[[3,[12,[19]]],3],[1],0,0,[[],20],[[],4],[[],6],[[],6],[1,8],[[3,8],3],[1,[[12,[5]]]],[[3,[12,[5]]],3],[1,[[12,[5]]]],[[3,[12,[5]]],3],[[],3],[[],7],[[4,4],[[12,[9]]]],[[4,6],[[12,[9]]]],[[6,6],[[12,[9]]]],[[6,4],[[12,[9]]]],[[8,8],[[12,[9]]]],[[7,7],[[12,[9]]]],[20,[[16,[15]]]],[20,[[16,[15]]]],[6],[6],[1,5],[[3,5],3],[8,5],[[7,5],7],[6,[[12,[4]]]],[4,6],0,[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],16],[[],21],[[],21],[[],21],[[],21],[[],21],[[],21],[[],21],[[],21],0],"c":[],"p":[[3,"Record"],[3,"Arguments"],[3,"RecordBuilder"],[4,"Level"],[15,"str"],[4,"LevelFilter"],[3,"MetadataBuilder"],[3,"Metadata"],[4,"Ordering"],[15,"bool"],[3,"ParseLevelError"],[4,"Option"],[3,"Formatter"],[6,"Result"],[3,"SetLoggerError"],[4,"Result"],[8,"Hasher"],[8,"Iterator"],[15,"u32"],[8,"Log"],[3,"TypeId"]]}\
}');
if (typeof window !== 'undefined' && window.initSearch) {window.initSearch(searchIndex)};
if (typeof exports !== 'undefined') {exports.searchIndex = searchIndex};