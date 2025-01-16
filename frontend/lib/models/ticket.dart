import 'package:tinytickets/models/comment.dart';

import 'crud.dart';

class Ticket extends Serialisable {
  int id;
  int assetId;
  DateTime time;
  String title;
  String creator;
  String creatorMail;
  String creatorPhone;
  String description;
  bool isClosed;
  List<Comment> comments = [];

  Ticket(
      {required this.id,
      required this.assetId,
      required this.time,
      required this.title,
      required this.creator,
      required this.creatorMail,
      required this.creatorPhone,
      required this.description,
      required this.isClosed,
      this.comments = const []});

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'asset_id': assetId,
      'time': time.toIso8601String(),
      'title': title,
      'creator': creator,
      'creator_mail': creatorMail,
      'creator_phone': creatorPhone,
      'description': description,
      'is_closed': isClosed
    };
  }

  factory Ticket.fromJson(Map<String, dynamic> json) {
    return Ticket(
        id: json['id'],
        assetId: json['asset_id'],
        time: json['time'] != null
            ? DateTime.parse(json['time'])
            : DateTime.now(),
        title: json['title'],
        creator: json['creator'],
        creatorMail: json['creator_mail'],
        creatorPhone: json['creator_phone'],
        description: json['description'],
        isClosed: json['is_closed'],
        comments: json['comments'] == null
            ? []
            : (json['comments'] as List)
                .map((e) => Comment.fromJson(e))
                .toList());
  }

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;
    return other is Ticket &&
        other.id == id &&
        other.assetId == assetId &&
        other.time == time &&
        other.title == title &&
        other.creator == creator &&
        other.creatorMail == creatorMail &&
        other.creatorPhone == creatorPhone &&
        other.description == description &&
        other.isClosed == isClosed;
  }

  @override
  int get hashCode {
    return Object.hash(
      id,
      assetId,
      time,
      title,
      creator,
      creatorMail,
      creatorPhone,
      description,
      isClosed,
    );
  }
}
