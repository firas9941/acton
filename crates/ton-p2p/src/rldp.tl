fec.raptorQ data_size:int symbol_size:int symbols_count:int = fec.Type;

rldp2.messagePart transfer_id:int256 fec_type:fec.Type part:int total_size:long seqno:int data:bytes = rldp2.MessagePart;
rldp2.confirm transfer_id:int256 part:int max_seqno:int received_mask:int received_count:int = rldp2.MessagePart;
rldp2.complete transfer_id:int256 part:int = rldp2.MessagePart;

rldp.query query_id:int256 max_answer_size:long timeout:int data:bytes = rldp.Message;
rldp.answer query_id:int256 data:bytes = rldp.Message;
