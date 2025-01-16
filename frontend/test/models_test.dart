// Import the test package and Counter class
import 'dart:convert';

import 'package:flutter_test/flutter_test.dart';
import 'package:tinytickets/models/asset.dart';
import 'package:tinytickets/models/ticket.dart';
import 'package:tinytickets/models/comment.dart';

void main() {
  group('Serialization', () {
    test(
        'Converting an Asset to json an retrieving it should give the same Asset',
        () async {
      final Asset a1 =
          Asset(id: 1, title: "test title", description: "test description");
      final a1Json = jsonEncode(a1.toJson());
      final a2 = Asset.fromJson(json.decode(a1Json));
      expect(a1, a2);
    });

    test(
        'Converting a Ticket to json an retrieving it should give the same Ticket',
        () async {
      final Ticket t1 = Ticket(
          id: 1,
          assetId: 1,
          time: DateTime.now(),
          title: "a title",
          creator: "a creator",
          creatorMail: "a mail",
          creatorPhone: "a tel",
          description: "a description",
          isClosed: false);
      final a1Json = jsonEncode(t1.toJson());
      final t2 = Ticket.fromJson(json.decode(a1Json));
      expect(t1, t2);
    });

    test(
        'Converting a Comment to json an retrieving it should give the same Comment',
        () async {
      final Comment c1 = Comment(
          id: 1,
          ticketId: 1,
          time: DateTime.now(),
          creator: "a creator",
          content: "a content");
      final a1Json = jsonEncode(c1.toJson());
      final c2 = Comment.fromJson(json.decode(a1Json));
      expect(c1, c2);
    });
  });
}
