import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:tinytickets/models/comment.dart';
import 'package:tinytickets/models/ticket.dart';

import '../globals.dart';
import 'asset.dart';

dynamic fromJSONbyType(Type t, Map<String, dynamic> map) {
  switch (t) {
    case Ticket:
      return Ticket.fromJson(map);
    case Asset:
      return Asset.fromJson(map);
    case Comment:
      return Comment.fromJson(map);
  }
}

String routeByType(Type t) {
  switch (t) {
    case Ticket:
      return "tickets";
    case Asset:
      return "assets";
    case Comment:
      return "comments";
    default:
      return "";
  }
}

abstract class Serialisable {
  Serialisable() {}
  fromJson(Map<String, dynamic> json) {}
  final int id = 0;
  Map<String, dynamic> toJson();
}

abstract class Crud<T extends Serialisable> {
  Create(T val) {}

  Read(int id) {}

  ReadAll() {}

  Update(T val) {}

  Delete(int id) {}
}

class MockCrud<T extends Serialisable> extends Crud<T> {
  List<T> _db = [];

  Create(T val) {
    _db.add(val);
  }

  Read(int id) {
    return _db[id];
  }

  ReadAll() {
    return _db;
  }

  Update(T val) {
    _db[val.id] = val;
  }

  Delete(int id) {
    _db.removeAt(id);
  }
}

class APICrud<T extends Serialisable> extends Crud<T> {
  final String route = routeByType(T);

  String get base => (App().prefs.getString("hostname") ?? "") + "/api";
  String get token => App().prefs.getString("token") ?? "";

  Future<T> Create(T val) async {
    final response = await http.post(
      Uri.parse('$base/$route'),
      headers: <String, String>{
        'Content-Type': 'application/json; charset=UTF-8',
        'X-TOKEN': token
      },
      body: jsonEncode(val),
    );
    if (response.statusCode != 201) {
      throw Exception(response.body.toString());
    } else {
      return fromJSONbyType(T, json.decode(utf8.decode(response.bodyBytes)));
    }
  }

  Future<T> Read(int id) async {
    final response = await http.get(
      Uri.parse('$base/$route/${id.toString()}'),
      headers: <String, String>{
        'X-TOKEN': token,
        'Content-Type': 'application/json'
      },
    );
    if (response.statusCode == 200) {
      return fromJSONbyType(T, json.decode(utf8.decode(response.bodyBytes)));
    } else {
      throw Exception('Failed to load object');
    }
  }

  Future<List<T>> ReadAll() async {
    final response = await http.get(
      Uri.parse('$base/$route/all'),
      headers: <String, String>{'X-TOKEN': token},
    );
    if (response.statusCode == 200) {
      final List t = json.decode(utf8.decode(response.bodyBytes));
      final List<T> list = t.map((e) => fromJSONbyType(T, e) as T).toList();
      return list;
    } else {
      throw Exception('Failed to load objects');
    }
  }

  Update(T val) async {
    final response = await http.patch(
      Uri.parse('$base/$route/${val.id}'),
      headers: <String, String>{
        'Content-Type': 'application/json; charset=UTF-8',
        'X-TOKEN': token
      },
      body: jsonEncode(val),
    );
    if (response.statusCode != 201) {
      throw Exception(response.body.toString());
    }
  }

  Delete(int id) async {
    final response = await http.delete(
      Uri.parse('$base/$route/${id}'),
      headers: <String, String>{'X-TOKEN': token},
    );
    if (response.statusCode != 200) {
      throw Exception(response.body.toString());
    }
  }
}
