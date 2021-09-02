import 'package:tinytickets/models/crud.dart';
import 'package:equatable/equatable.dart';

class Asset extends Serialisable with EquatableMixin {
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
  List<Object> get props {
    return [id, title, description];
  }

  @override
  bool get stringify => true;
}
