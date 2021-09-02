import 'package:equatable/equatable.dart';

import 'crud.dart';

class Comment extends Serialisable with EquatableMixin {
  int id;
  int ticket_id;
  DateTime time;
  String content;

  Comment(
      {required this.id,
      required this.ticket_id,
      required this.time,
      required this.content});

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'ticket_id': ticket_id,
      'time': time.toIso8601String(),
      'content': content
    };
  }

  factory Comment.fromJson(Map<String, dynamic> json) {
    return Comment(
        id: json['id'],
        ticket_id: json['ticket_id'],
        time: json['time'] != null
            ? DateTime.parse(json['time'])
            : DateTime.now(),
        content: json['content']);
  }

  @override
  List<Object> get props {
    return [id, ticket_id, time, content];
  }

  @override
  bool get stringify => true;
}
