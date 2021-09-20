import 'package:equatable/equatable.dart';
import 'package:tinytickets/models/comment.dart';

import 'crud.dart';

class Ticket extends Serialisable with EquatableMixin {
  int id;
  int asset_id;
  DateTime time;
  String title;
  String creator;
  String creator_mail;
  String creator_phone;
  String description;
  bool is_closed;
  List<Comment> comments = [];

  Ticket(
      {required this.id,
      required this.asset_id,
      required this.time,
      required this.title,
      required this.creator,
      required this.creator_mail,
      required this.creator_phone,
      required this.description,
      required this.is_closed,
      this.comments = const []});

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'asset_id': asset_id,
      'time': time.toIso8601String(),
      'title': title,
      'creator': creator,
      'creator_mail': creator_mail,
      'creator_phone': creator_phone,
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
        creator: json['creator'],
        creator_mail: json['creator_mail'],
        creator_phone: json['creator_phone'],
        description: json['description'],
        is_closed: json['is_closed'],
        comments: json['comments'] == null
            ? []
            : (json['comments'] as List)
                .map((e) => Comment.fromJson(e))
                .toList());
  }

  @override
  List<Object> get props {
    return [
      id,
      asset_id,
      time,
      title,
      creator,
      creator_mail,
      creator_phone,
      description,
      is_closed,
    ];
  }

  @override
  bool get stringify => true;
}
