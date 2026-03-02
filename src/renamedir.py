#!/usr/bin/env python3
# -*- coding: utf-8 -*-

import os
import re
import sys
import argparse
from pathlib import Path


def camel_to_snake(name):
    """
    将驼峰命名转换为蛇形命名
    例如: SomeTypes -> some_types, Cache -> cache, XMLParser -> xml_parser
    """
    # 处理特殊情况：全大写缩写
    name = re.sub(r'(?<=[a-z])([A-Z])', r'_\1', name)  # 小写后跟大写：someTypes -> some_Types
    name = re.sub(r'([A-Z]+)([A-Z][a-z])', r'\1_\2', name)  # 连续大写后跟大写小写：XMLParser -> XML_Parser
    name = re.sub(r'([a-z])([A-Z])', r'\1_\2', name)  # 小写后跟大写：someTypes -> some_Types
    
    # 将连续的大写转换为小写，但保持单词边界
    parts = name.split('_')
    for i, part in enumerate(parts):
        if part.isupper() and len(part) > 1:
            parts[i] = part.lower()
        else:
            parts[i] = part.lower()
    
    return '_'.join(parts)


def is_snake_case(name):
    """
    判断一个名称是否符合蛇形命名规范
    蛇形命名规则：全小写，单词之间用下划线分隔，不能以下划线开头或结尾
    """
    # 检查是否为空
    if not name:
        return False
    
    # 检查是否以下划线开头或结尾
    if name.startswith('_') or name.endswith('_'):
        return False
    
    # 检查是否包含大写字母
    if any(c.isupper() for c in name):
        return False
    
    # 检查是否包含非字母数字和下划线的字符
    if not re.match(r'^[a-z0-9_]+$', name):
        return False
    
    # 检查是否有连续的下划线
    if '__' in name:
        return False
    
    # 检查是否有有效的单词分隔（可选）
    # 可以允许纯数字，但通常命名不会纯数字
    return True


def should_skip_directory(dirpath, skip_hidden=True, skip_git=True):
    """
    判断是否应该跳过该目录
    """
    dir_name = os.path.basename(dirpath)
    
    # 跳过隐藏目录
    if skip_hidden and dir_name.startswith('.'):
        return True
    
    # 跳过常见的版本控制目录
    if skip_git and dir_name in ['.git', '.svn', '.hg']:
        return True
    
    # 跳过Python缓存目录
    if dir_name in ['__pycache__', 'node_modules']:
        return True
    
    return False


def rename_directory(old_path, new_path, dry_run=False):
    """
    重命名目录，如果是dry_run则只打印不执行
    """
    if dry_run:
        print(f"[DRY RUN] 将重命名: {old_path} -> {new_path}")
        return True
    else:
        try:
            os.rename(old_path, new_path)
            print(f"已重命名: {old_path} -> {new_path}")
            return True
        except OSError as e:
            print(f"重命名失败 {old_path}: {e}", file=sys.stderr)
            return False


def process_directory(root_path, recursive=True, dry_run=False, exclude_patterns=None):
    """
    处理目录下的所有子目录
    """
    if exclude_patterns is None:
        exclude_patterns = []
    
    root_path = os.path.abspath(root_path)
    
    if recursive:
        # 递归遍历所有子目录
        for dirpath, dirnames, _ in os.walk(root_path, topdown=True):
            # 过滤掉要跳过的目录
            dirnames[:] = [d for d in dirnames 
                          if not should_skip_directory(os.path.join(dirpath, d))]
            
            for dirname in dirnames:
                full_path = os.path.join(dirpath, dirname)
                
                # 检查是否匹配排除模式
                if any(pattern in full_path for pattern in exclude_patterns):
                    continue
                
                # 如果不是蛇形命名，则重命名
                if not is_snake_case(dirname):
                    new_name = camel_to_snake(dirname)
                    if new_name != dirname:
                        new_path = os.path.join(dirpath, new_name)
                        rename_directory(full_path, new_path, dry_run)
    else:
        # 只处理根目录下的直接子目录
        try:
            for item in os.listdir(root_path):
                full_path = os.path.join(root_path, item)
                if os.path.isdir(full_path) and not should_skip_directory(full_path):
                    if not is_snake_case(item):
                        new_name = camel_to_snake(item)
                        if new_name != item:
                            new_path = os.path.join(root_path, new_name)
                            rename_directory(full_path, new_path, dry_run)
        except OSError as e:
            print(f"无法读取目录 {root_path}: {e}", file=sys.stderr)


def main():
    parser = argparse.ArgumentParser(
        description='将目录名转换为蛇形命名（snake_case）',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
示例:
  %(prog)s /path/to/dir                 # 递归处理指定目录下的所有子目录
  %(prog)s /path/to/dir --non-recursive # 只处理直接子目录
  %(prog)s /path/to/dir --dry-run       # 预览将要进行的更改
  %(prog)s dir1 dir2 dir3                # 同时处理多个目录
        """
    )
    
    parser.add_argument(
        'paths',
        nargs='+',
        help='要处理的目录路径'
    )
    
    parser.add_argument(
        '-r', '--recursive',
        action='store_true',
        default=True,
        help='递归处理所有子目录（默认启用）'
    )
    
    parser.add_argument(
        '-n', '--non-recursive',
        dest='recursive',
        action='store_false',
        help='只处理直接子目录，不递归'
    )
    
    parser.add_argument(
        '--dry-run',
        action='store_true',
        help='预览模式，只显示将要进行的更改但不实际执行'
    )
    
    parser.add_argument(
        '--exclude',
        metavar='PATTERN',
        action='append',
        default=[],
        help='排除包含指定模式的路径（可多次使用）'
    )
    
    parser.add_argument(
        '--no-skip-hidden',
        dest='skip_hidden',
        action='store_false',
        default=True,
        help='不跳过隐藏目录'
    )
    
    args = parser.parse_args()
    
    # 处理每个给定的路径
    for path_str in args.paths:
        path = Path(path_str)
        
        if not path.exists():
            print(f"错误: 路径不存在: {path}", file=sys.stderr)
            continue
        
        if not path.is_dir():
            print(f"错误: 路径不是目录: {path}", file=sys.stderr)
            continue
        
        print(f"\n处理目录: {path}")
        process_directory(
            str(path),
            recursive=args.recursive,
            dry_run=args.dry_run,
            exclude_patterns=args.exclude
        )


if __name__ == '__main__':
    main()

