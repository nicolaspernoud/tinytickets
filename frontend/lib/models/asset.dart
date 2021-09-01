import 'package:tinytickets/models/crud.dart';

class Asset extends Serialisable {
  int id;
  String title;
  String description;

  Asset({
    required this.id,
    required this.title,
    required this.description,
  });

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'title': title,
      'description': description,
    };
  }

  factory Asset.fromJson(Map<String, dynamic> data) {
    return Asset(
      id: data['id'],
      title: data['title'],
      description: data['description'],
    );
  }
}
