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

  @override
  bool operator ==(Object other) {
    if (identical(this, other)) return true;
    if (other.runtimeType != runtimeType) return false;
    return other is Asset &&
        other.id == id &&
        other.title == title &&
        other.description == description;
  }

  @override
  int get hashCode {
    return Object.hash(id, title, description);
  }
}
