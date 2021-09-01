import 'package:tinytickets/models/comment.dart';

import 'crud.dart';

class Ticket extends Serialisable {
  int id;
  int asset_id;
  DateTime time;
  String title;
  String description;
  bool is_closed;
  List<Comment> comments = [];

  Ticket(
      {required this.id,
      required this.asset_id,
      required this.time,
      required this.title,
      required this.description,
      required this.is_closed,
      this.comments = const []});

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'asset_id': asset_id,
      'time': time.toIso8601String(),
      'title': title,
      'description': description,
      'is_closed': is_closed
    };
  }

  factory Ticket.fromJson(Map<String, dynamic> json) {
    return Ticket(
        id: json['id'],
        asset_id: json['asset_id'],
        time: json['time'] != null
            ? DateTime.parse(json['time'])
            : DateTime.now(),
        title: json['title'],
        description: json['description'],
        is_closed: json['is_closed'],
        comments: json['comments'] == null
            ? []
            : (json['comments'] as List)
                .map((e) => Comment.fromJson(e))
                .toList());
  }
}
