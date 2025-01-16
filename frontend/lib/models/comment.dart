import 'crud.dart';

class Comment extends Serialisable {
  int id;
  int ticketId;
  DateTime time;
  String creator;
  String content;

  Comment(
      {required this.id,
      required this.ticketId,
      required this.time,
      required this.creator,
      required this.content});

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'ticket_id': ticketId,
      'time': time.toIso8601String(),
      'creator': creator,
      'content': content
    };
  }

  factory Comment.fromJson(Map<String, dynamic> json) {
    return Comment(
        id: json['id'],
        ticketId: json['ticket_id'],
        time: json['time'] != null
            ? DateTime.parse(json['time'])
            : DateTime.now(),
        creator: json['creator'],
        content: json['content']);
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;
    return other is Comment &&
        other.id == id &&
        other.ticketId == ticketId &&
        other.time == time &&
        other.creator == creator &&
        other.content == content;
  }

  @override
  int get hashCode {
    return Object.hash(id, ticketId, time, creator, content);
  }
}
